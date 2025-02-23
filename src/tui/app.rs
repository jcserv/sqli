use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use tui_textarea::TextArea;
use tui_tree_widget::{TreeItem, TreeState};

use crate::collection::{CollectionScope, SelectedFile};
use crate::config::ConfigManager;
use crate::file::{self, parse_selected_file};
use crate::file_operations::{create_file_or_folder, rename_file_or_folder};
use crate::query::{self, execute_query};
use crate::sql::interface::QueryResult;

use super::modal::{ModalEvent, ModalManager, ModalType};
use super::navigation::{NavigationManager, PaneId};
use super::ui::UI;
use super::widgets::edit_file_modal::EditFileModal;
use super::widgets::modal::ModalAction;
use super::widgets::new_file_modal::NewFileModal;
use super::widgets::password_modal::PasswordModal;
use super::widgets::searchable_textarea::SearchableTextArea;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Password,
    NewFile,
    EditFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    None,
    ExecuteQuery,
    SaveQuery,
    CreateFile,
    EditFile,
}

#[derive(Debug)]
pub struct AsyncCommandResult {
    pub command: AppCommand,
    pub message: Option<String>,
}

impl AsyncCommandResult {
    pub fn new(command: AppCommand) -> Self {
        Self {
            command,
            message: None,
        }
    }

    pub fn with_message(command: AppCommand, message: String) -> Self {
        Self {
            command,
            message: Some(message),
        }
    }
}

// UI-specific state
pub struct UIState<'a> {
    pub message: String,
    pub collection_state: TreeState<String>,
    pub collection_items: Vec<TreeItem<'a, String>>,
    pub workspace: SearchableTextArea<'a>,
    pub search: SearchBox<'a>,
}

// Handles the search interface state
pub struct SearchBox<'a> {
    pub textarea: TextArea<'a>,
    pub open: bool,
    pub replace_mode: bool,
}

impl Default for SearchBox<'_> {
    fn default() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
        );
        Self {
            textarea,
            open: false,
            replace_mode: false,
        }
    }
}

// Query-related state
pub struct QueryState {
    pub selected_connection: Option<String>,
    pub available_connections: Vec<String>, 
    pub current_password: Option<String>, 
    pub query_result: QueryResult,
    pub pending_command: AppCommand,
    pub pending_async_operation: Option<tokio::task::JoinHandle<AsyncCommandResult>>,
}

#[derive(Debug)]
pub struct SelectedFileInfo {
    pub name: String,
    pub is_folder: bool,
    pub scope: CollectionScope,
}

#[derive(Debug)]
pub enum FileOperationState {
    Create {
        name: String,
        is_folder: bool,
        scope: CollectionScope,
    },
    Edit {
        name: String,
        scope: CollectionScope,
    },
}

pub struct App<'a> {
    pub mode: Mode,
    pub should_quit: bool,
    
    pub ui_state: UIState<'a>,
    pub query_state: QueryState,
    pub navigation: NavigationManager,
    pub modal_manager: ModalManager,
    pub file_operation_state: Option<FileOperationState>,
}

// Core application initialization and state
impl<'a> App<'a> {
    pub fn new() -> Self {
        let mut workspace = SearchableTextArea::default();
        workspace.init();

        let collection_items = super::panes::collections::CollectionsPane::load_collections();
        
        let mut navigation = NavigationManager::new();
        navigation.register_pane(PaneId::Header, 1);
        navigation.register_pane(PaneId::Collections, 1);
        navigation.register_pane(PaneId::Workspace, 1);
        navigation.register_pane(PaneId::Results, 1);

        let mut app = Self {
            mode: Mode::Normal,
            should_quit: false,
            
            ui_state: UIState {
                message: String::new(),
                collection_state: TreeState::default(),
                collection_items,
                workspace,
                search: SearchBox::default(),
            },
            
            query_state: QueryState {
                selected_connection: None,
                available_connections: Vec::new(),
                current_password: None,
                query_result: QueryResult::default(),
                pending_command: AppCommand::None,
                pending_async_operation: None,
            },
            
            navigation,
            modal_manager: ModalManager::new(),
            file_operation_state: None,
        };
        
        if let Err(e) = app.load_connections() {
            app.ui_state.message = format!("Error loading connections: {}", e);
        }

        app
    }

    pub fn is_header_active(&self) -> bool {
        self.navigation.is_active(PaneId::Header)
    }

    pub fn is_collections_active(&self) -> bool {
        self.navigation.is_active(PaneId::Collections)
    }
    
    pub fn is_workspace_active(&self) -> bool {
        self.navigation.is_active(PaneId::Workspace)
    }
    
    pub fn is_results_active(&self) -> bool {
        self.navigation.is_active(PaneId::Results)
    }

    pub fn is_edit_mode(&self) -> bool {
        [PaneId::Collections, PaneId::Workspace, PaneId::Results]
            .iter()
            .any(|&pane_id| self.is_pane_in_edit_mode(pane_id))
    }
    
    pub fn is_pane_in_edit_mode(&self, pane_id: PaneId) -> bool {
        if let Some(info) = self.navigation.get_pane_info(pane_id) {
            info.is_editing()
        } else {
            false
        }
    }
}

// Connection management
impl<'a> App<'a> {
    pub fn load_connections(&mut self) -> Result<()> {
        let config_manager = ConfigManager::new()?;
        let connections = config_manager.list_connections()?;
        
        if !connections.is_empty() {
            self.query_state.available_connections = connections;
            if self.query_state.selected_connection.is_none() {
                self.query_state.selected_connection = Some(self.query_state.available_connections[0].clone());
            }
        }
        
        Ok(())
    }

    fn reload_collections(&mut self) {
        self.ui_state.collection_items = super::panes::collections::CollectionsPane::load_collections();
    }

    pub fn next_connection(&mut self) {
        if self.query_state.available_connections.is_empty() {
            return;
        }

        let current_idx = self.query_state.selected_connection
            .as_ref()
            .and_then(|current| self.query_state.available_connections.iter().position(|x| x == current))
            .unwrap_or(0);

        let next_idx = (current_idx + 1) % self.query_state.available_connections.len();
        self.query_state.selected_connection = Some(self.query_state.available_connections[next_idx].clone());
    }

    pub fn previous_connection(&mut self) {
        if self.query_state.available_connections.is_empty() {
            return;
        }

        let current_idx = self.query_state.selected_connection
            .as_ref()
            .and_then(|current| self.query_state.available_connections.iter().position(|x| x == current))
            .unwrap_or(0);

        let prev_idx = if current_idx == 0 {
            self.query_state.available_connections.len() - 1
        } else {
            current_idx - 1
        };
        
        self.query_state.selected_connection = Some(self.query_state.available_connections[prev_idx].clone());
    }

    pub fn get_current_connection(&self) -> Option<String> {
        self.query_state.selected_connection.clone()
    }
}

// Query execution and management
impl<'a> App<'a> {
    pub fn check_and_execute_query(&mut self) {
        if let Some(conn_name) = &self.query_state.selected_connection {
            match query::get_connection(conn_name) {
                Ok(Some(conn)) => {
                    if let Some(pwd) = &self.query_state.current_password {
                        self.execute_query_with_password(Some(pwd.clone()));
                        return;
                    }
                    if conn.requires_password() {
                        self.show_password_prompt();
                        return;
                    }
                }
                Ok(None) => {
                    self.ui_state.message = format!("Connection '{}' not found", conn_name);
                    return;
                }
                Err(e) => {
                    self.ui_state.message = format!("Error checking connection: {}", e);
                    return;
                }
            }
        }
        self.execute_query_with_password(None);
    }

    fn execute_query_with_password(&mut self, password: Option<String>) {
        let sql = self.ui_state.workspace.get_content();
        let connection = self.query_state.selected_connection.clone();

        let handle = tokio::spawn(async move {
            match execute_query(sql, None, connection, password).await {
                Ok(_) => AsyncCommandResult::new(AppCommand::ExecuteQuery),
                Err(e) => AsyncCommandResult::with_message(
                    AppCommand::ExecuteQuery,
                    format!("Query error: {}", e)
                ),
            }
        });
        
        self.query_state.pending_async_operation = Some(handle);
    }

    pub fn save_query(&mut self) {
        let content = self.ui_state.workspace.get_content();
        if content.is_empty() {
            self.ui_state.message = "Nothing to save".to_string();
            return;
        }

        let selected = self.ui_state.collection_state.selected();
        if selected.is_empty() {
            self.ui_state.message = "No file selected".to_string();
            return;
        }

        let selected_file = match parse_selected_file(&selected) {
            Some(file) => file,
            None => {
                self.ui_state.message = "Invalid selection".to_string();
                return;
            }
        };

        match file::save_file(&selected_file, &content) {
            Ok(_) => {
                self.ui_state.message = "File saved successfully".to_string();
            },
            Err(e) => {
                self.ui_state.message = format!("Error saving file: {}", e);
            }
        }
    }
}

// Event handling
impl<'a> App<'a> {
    pub fn tick(&mut self) {
        if self.query_state.pending_command != AppCommand::None {
            match self.query_state.pending_command {
                AppCommand::ExecuteQuery => {
                    self.check_and_execute_query();
                },
                AppCommand::SaveQuery => {
                    self.save_query();
                },
                AppCommand::CreateFile => {
                    if let Some(FileOperationState::Create { name, is_folder, scope }) = &self.file_operation_state {
                        match create_file_or_folder(name, *is_folder, *scope) {
                            Ok(_) => {
                                self.ui_state.message = format!("{} created successfully", 
                                    if *is_folder { "Folder" } else { "File" });
                                self.reload_collections();
                            },
                            Err(e) => {
                                self.ui_state.message = format!("Error creating {}: {}", 
                                    if *is_folder { "folder" } else { "file" }, e);
                            }
                        }
                    }
                },
                AppCommand::EditFile => {
                    if let Some(FileOperationState::Edit { name, scope }) = &self.file_operation_state {
                        if let Some(old_info) = self.get_selected_file_info() {
                            match rename_file_or_folder(
                                &old_info.name, 
                                name, 
                                old_info.scope,
                                *scope
                            ) {
                                Ok(_) => {
                                    self.ui_state.message = "File/folder updated successfully".to_string();
                                    self.reload_collections();
                                },
                                Err(e) => {
                                    self.ui_state.message = format!("Error updating file/folder: {}", e);
                                }
                            }
                        }
                    }
                },
                AppCommand::None => {},
            }
            
            self.query_state.pending_command = AppCommand::None;
        }
    }

    pub fn handle_key(&mut self, ui: &mut UI, key_event: KeyEvent) -> Result<bool> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(true);
            }
            (KeyCode::Tab, _) if !self.is_edit_mode() && !self.modal_manager.is_modal_active() => {
                _ = self.navigation.cycle_pane(false);
                return Ok(false);
            }
            (KeyCode::Char('n'), KeyModifiers::CONTROL) if !self.modal_manager.is_modal_active() => {
                self.show_new_file_modal();
                return Ok(false);
            }
            (KeyCode::Char('e'), KeyModifiers::CONTROL) if !self.modal_manager.is_modal_active() => {
                if let Some(selected_file) = self.get_selected_file_info() {
                    self.show_edit_file_modal(selected_file.name, selected_file.is_folder, selected_file.scope);
                }
                return Ok(false);
            }
            _ => {
                if self.modal_manager.is_modal_active() {
                    self.handle_modal_key_event(key_event)?;
                    return Ok(false);
                }
                
                return ui.handle_key_event(self, key_event);
            }
        }
    }

    pub fn handle_mouse(&mut self, ui: &mut super::ui::UI, mouse_event: MouseEvent) -> Result<bool> {
        if self.modal_manager.is_modal_active() {
            let terminal_area = crossterm::terminal::size()
                .map(|(w, h)| Rect::new(0, 0, w, h))
                .unwrap_or(Rect::new(0, 0, 80, 24));

            self.handle_modal_mouse_event(mouse_event, terminal_area)?;
            return Ok(false);
        }
        ui.handle_mouse_event(self, mouse_event)
    }
}

// Modal handling
impl<'a> App<'a> {
    fn handle_modal_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.modal_manager.handle_event(ModalEvent::Key(key_event))? {
            ModalAction::Close => {
                self.close_modal();
            }
            ModalAction::Custom(action) => {
                match action.as_str() {
                    "submit" => {
                        if let Some(modal) = self.modal_manager.get_active_modal_as::<PasswordModal>() {
                            self.query_state.current_password = modal.get_password();
                            self.execute_query_with_password(self.query_state.current_password.clone());
                        }
                        self.close_modal();
                    }
                    "cancel" => {
                        self.close_modal();
                    }
                    _ => {}
                }
            }
            ModalAction::None => {}
        }
        Ok(())
    }

    fn handle_modal_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<()> {
        match self.modal_manager.handle_event(ModalEvent::Mouse(mouse_event, area))? {
            ModalAction::Close => {
                self.close_modal();
            }
            ModalAction::Custom(action) => {
                match action.as_str() {
                    "new" => {
                        if let Some(modal) = self.modal_manager.get_active_modal_as::<NewFileModal>() {
                            let (name, file_type, scope) = modal.get_values();
                            
                            if name.is_empty() {
                                self.ui_state.message = "Name cannot be empty".to_string();
                                return Ok(());
                            }
                            if file_type == "file" && !name.ends_with(".sql") {
                                self.ui_state.message = "File name must end with .sql".to_string();
                                return Ok(());
                            }
                            self.query_state.pending_command = AppCommand::CreateFile;
                            
                            self.file_operation_state = Some(FileOperationState::Create {
                                name,
                                is_folder: file_type == "folder",
                                scope,
                            });
                        }
                    }
                    "edit" => {
                        if let Some(modal) = self.modal_manager.get_active_modal_as::<EditFileModal>() {
                            let (name, scope) = modal.get_values();
                            
                            if name.is_empty() {
                                self.ui_state.message = "Name cannot be empty".to_string();
                                return Ok(());
                            }

                            self.query_state.pending_command = AppCommand::EditFile;
                            
                            self.file_operation_state = Some(FileOperationState::Edit {
                                name,
                                scope,
                            });
                        }
                    }
                    "submit" => {
                        if let Some(modal) = self.modal_manager.get_active_modal_as::<PasswordModal>() {
                            self.query_state.current_password = modal.get_password();
                            self.execute_query_with_password(self.query_state.current_password.clone());
                        }
                        self.close_modal();
                    }
                    "cancel" => {
                        self.close_modal();
                    }
                    _ => {}
                }
            }
            ModalAction::None => {}
        }
        Ok(())
    }

    fn close_modal(&mut self) {
        self.modal_manager.close_modal();
        self.mode = Mode::Normal;
    }

    pub fn show_password_prompt(&mut self) {
        self.modal_manager.show_modal(ModalType::Password);
        self.mode = Mode::Password;
    }

    pub fn show_new_file_modal(&mut self) {
        self.modal_manager.show_modal(ModalType::NewFile);
        self.mode = Mode::NewFile;
    }

    pub fn show_edit_file_modal(&mut self, name: String, is_folder: bool, current_scope: CollectionScope) {
        self.modal_manager.show_modal(ModalType::EditFile {
            name,
            is_folder,
            current_scope,
        });
        self.mode = Mode::EditFile;
    }

    fn get_selected_file_info(&self) -> Option<SelectedFileInfo> {
        let selected = self.ui_state.collection_state.selected();
        if selected.is_empty() {
            return None;
        }

        if let Some(file) = parse_selected_file(&selected) {
            match file {
                SelectedFile::Config(scope) => Some(SelectedFileInfo {
                    name: "config.yaml".to_string(),
                    is_folder: false,
                    scope,
                }),
                SelectedFile::Sql { collection, filename, scope } => {
                    let is_folder = filename.is_empty();
                    let name = if is_folder { collection } else { filename };
                    Some(SelectedFileInfo {
                        name,
                        is_folder,
                        scope,
                    })
                }
            }
        } else {
            None
        }
    }
}

// Async operation handling
impl<'a> App<'a> {
    pub fn process_async_results(&mut self) {
        if let Some(handle) = &mut self.query_state.pending_async_operation {
            if handle.is_finished() {
                let handle = std::mem::take(&mut self.query_state.pending_async_operation).unwrap();
                
                match tokio::task::block_in_place(|| futures::executor::block_on(async {
                    handle.await
                })) {
                    Ok(result) => {
                        match result.command {
                            AppCommand::ExecuteQuery => {
                                let sql = self.ui_state.workspace.get_content();
                                let connection = self.query_state.selected_connection.clone();
                                match tokio::task::block_in_place(|| {
                                    futures::executor::block_on(async {
                                        execute_query(
                                            sql,
                                            None,
                                            connection,
                                            self.query_state.current_password.clone(),
                                        ).await
                                    })
                                }) {
                                    Ok(query_result) => {
                                        self.query_state.query_result = query_result;
                                        self.ui_state.message = format!(
                                            "Query executed successfully in {}ms",
                                            self.query_state.query_result.execution_time.as_millis()
                                        );
                                    }
                                    Err(e) => {
                                        if e.to_string().contains("password authentication failed") {
                                            self.show_password_prompt();
                                        } else {
                                            self.ui_state.message = format!("Error executing query: {}", e);
                                            self.query_state.query_result = QueryResult::empty();
                                        }
                                    }
                                }
                            },
                            AppCommand::SaveQuery => {
                                self.save_query();
                            },
                            _ => {},
                        }
    
                        if let Some(msg) = result.message {
                            self.ui_state.message = msg;
                        }
                    },
                    Err(e) => {
                        self.ui_state.message = format!("Error in async operation: {}", e);
                    }
                }
            }
        }
    }
}