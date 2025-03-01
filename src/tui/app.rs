use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use tui_textarea::TextArea;
use tui_tree_widget::{TreeItem, TreeState};

use crate::collection::{CollectionScope, SelectedFile};
use crate::config::CONFIG_FILE_NAME;
use crate::file::{get_selected_folder_context, parse_selected_file, FileSystem};
use crate::query::{self, execute_query};
use crate::settings::UserSettings;
use crate::sql::result::QueryResult;

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
    DeleteFile,
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
    pub collection_name: Option<String>,
    pub is_folder: bool,
    pub scope: CollectionScope,
}

#[derive(Debug)]
pub enum FileOperationState {
    Create {
        name: String,
        is_folder: bool,
        scope: CollectionScope,
        parent_folder: Option<String>,
    },
    Edit {
        name: String,
        scope: CollectionScope,
    },
    Delete { 
        name: String,
        is_folder: bool,
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
    pub fs: FileSystem,
}

// Core application initialization and state
impl App<'_> {
    pub fn new() -> Result<Self> {
        Self::with_settings(None)
    }

    pub fn with_settings(settings: Option<UserSettings>) -> Result<Self> {
        let fs = match settings {
            Some(settings) => FileSystem::with_paths(settings.user_dir, settings.workspace_dir)?,
            None => FileSystem::new()?,
        };
        
        let mut workspace = SearchableTextArea::default();
        workspace.init();

        let collections = match crate::collection::load_collections(&fs) {
            Ok(collections) => collections,
            Err(e) => {
                eprintln!("Error loading collections: {}", e);
                Vec::new()
            }
        };
        
        let collection_items = crate::collection::build_collection_tree(&collections, &fs);
        
        let mut navigation = NavigationManager::new();
        navigation.register_pane(PaneId::Header, 2);
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
            fs,
        };
        
        if let Err(e) = app.load_connections() {
            app.ui_state.message = format!("Error loading connections: {}", e);
        }

        Ok(app)
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
        [PaneId::Header, PaneId::Collections, PaneId::Workspace, PaneId::Results]
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
impl App<'_> {
    pub fn load_connections(&mut self) -> Result<()> {
        let config_manager = crate::config::ConfigManager::with_filesystem(self.fs.clone());
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
        if let Ok(collections) = crate::collection::load_collections(&self.fs) {
            self.ui_state.collection_items = crate::collection::build_collection_tree(&collections, &self.fs);
        }
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
impl App<'_> {
    fn check_and_execute_query(&mut self) {
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

        let selected_file = match crate::file::parse_selected_file(selected) {
            Some(file) => file,
            None => {
                self.ui_state.message = "Invalid selection".to_string();
                return;
            }
        };

        match self.fs.save_file(&selected_file, &content) {
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
impl App<'_> {
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
                    self.handle_new();
                },
                AppCommand::EditFile => {
                    self.handle_edit();
                },
                AppCommand::DeleteFile => {
                    self.handle_delete();
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
                Ok(true)
            }
            (KeyCode::BackTab, _) if !self.is_edit_mode() && !self.modal_manager.is_modal_active() => {
                _ = self.navigation.cycle_pane(true);
                Ok(false)
            }
            (KeyCode::Tab, _) if !self.is_edit_mode() && !self.modal_manager.is_modal_active() => {
                _ = self.navigation.cycle_pane(false);
                Ok(false)
            }
            (KeyCode::Char('n'), KeyModifiers::CONTROL) if !self.modal_manager.is_modal_active() => {
                self.show_new_file_modal();
                Ok(false)
            }
            (KeyCode::Char('e'), KeyModifiers::CONTROL) if !self.modal_manager.is_modal_active() => {
                if let Some(selected_file) = self.get_selected_file_info() {
                    if selected_file.name.starts_with(CONFIG_FILE_NAME) {
                        return Ok(false);
                    }
                    self.show_edit_file_modal(selected_file.name, selected_file.is_folder, selected_file.scope);
                }
                Ok(false)
            }
            _ => {
                if self.modal_manager.is_modal_active() {
                    self.handle_modal_key_event(key_event)?;
                    return Ok(false);
                }
                
                ui.handle_key_event(self, key_event)
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
impl App<'_> {
    fn handle_modal_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.modal_manager.handle_event(ModalEvent::Key(key_event))? {
            ModalAction::Close => {
                self.close_modal();
            }
            ModalAction::Custom(action) => {
                match action.as_str() {
                    "new" => {
                        self.dispatch_new();
                    }
                    "edit" => {
                        self.dispatch_edit();
                    }
                    "delete" => {
                        self.dispatch_delete();
                    }
                    "submit" => {
                        self.handle_submit();
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
        if let Some(modal) = self.modal_manager.get_modal_mut() {
            match modal.handle_mouse_event(mouse_event, area)? {
                ModalAction::Close => {
                    self.close_modal();
                }
                ModalAction::Custom(action) => {
                    match action.as_str() {
                        "new" => {
                            self.dispatch_new();
                        }
                        "edit" => {
                            self.dispatch_edit();
                        }
                        "delete" => {
                            self.dispatch_delete();
                        }
                        "submit" => {
                            self.handle_submit();
                        }
                        "cancel" => {
                            self.close_modal();
                        }
                        _ => {}
                    }
                }
                ModalAction::None => {}
            }
        }
        Ok(())
    }

    fn close_modal(&mut self) {
        self.modal_manager.close_modal();
        self.mode = Mode::Normal;
    }

    fn handle_submit(&mut self) {
        if let Some(modal) = self.modal_manager.get_active_modal_as::<PasswordModal>() {
            self.query_state.current_password = modal.get_password();
            self.execute_query_with_password(self.query_state.current_password.clone());
        }
        self.close_modal();
    }

    fn dispatch_new(&mut self) {
        if let Some(modal) = self.modal_manager.get_active_modal_as::<NewFileModal>() {
            let (name, file_type, scope, parent_folder) = modal.get_values();
            
            if name.is_empty() {
                self.ui_state.message = "Name cannot be empty".to_string();
                return;
            }

            if file_type == "file" && !name.ends_with(".sql") {
                self.ui_state.message = "File name must end with .sql".to_string();
                return;
            }
            
            self.query_state.pending_command = AppCommand::CreateFile;

            self.file_operation_state = Some(FileOperationState::Create {
                name,
                is_folder: file_type == "folder",
                scope,
                parent_folder,
            });
        }
        self.close_modal();
    }

    fn handle_new(&mut self) {
        if let Some(FileOperationState::Create { name, is_folder, scope, parent_folder }) = &self.file_operation_state {    
            let path = match parent_folder {
                Some(folder) if !is_folder => {
                    format!("{}/{}", folder, name)
                },
                _ => name.clone(),
            };
            
            match self.fs.create_file_or_folder(&path, *is_folder, *scope) {
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
    }

    fn dispatch_edit(&mut self) {
        if let Some(modal) = self.modal_manager.get_active_modal_as::<EditFileModal>() {
            let (name, scope) = modal.get_values();
            
            if name.is_empty() {
                self.ui_state.message = "Name cannot be empty".to_string();
                return;
            }

            if name.starts_with(CONFIG_FILE_NAME) {
                return;
            }

            self.query_state.pending_command = AppCommand::EditFile;
            
            self.file_operation_state = Some(FileOperationState::Edit {
                name,
                scope,
            });
        }
        self.close_modal();
    }

    fn handle_edit(&mut self) {
        if let Some(FileOperationState::Edit { name, scope }) = &self.file_operation_state {
            if let Some(old_info) = self.get_selected_file_info() {
                if name == &old_info.name && scope == &old_info.scope {
                    self.ui_state.message = "No changes made".to_string();
                    return;
                }

                let old_path = if old_info.is_folder {
                    old_info.name.clone()
                } else {
                    match &old_info.collection_name {
                        Some(collection) => format!("{}/{}", collection, old_info.name),
                        None => old_info.name.clone()
                    }
                };
                
                let new_path = if old_info.is_folder {
                    name.clone()
                } else if let Some(collection) = &old_info.collection_name {
                    format!("{}/{}", collection, name)
                } else {
                    name.clone()
                };

                match self.fs.rename_file_or_folder(
                    &old_path,
                    &new_path,
                    old_info.scope,
                    *scope
                ) {
                    Ok(_) => {
                        self.ui_state.message = format!("{} renamed successfully", 
                            if old_info.is_folder { "Folder" } else { "File" });
                        self.reload_collections();
                    },
                    Err(e) => {
                        self.ui_state.message = format!("Error renaming {}: {}", 
                            if old_info.is_folder { "folder" } else { "file" }, e);
                    }
                }
            }
        }
    }

    fn dispatch_delete(&mut self) {
        if let Some(selected_file) = self.get_selected_file_info() {
            self.query_state.pending_command = AppCommand::DeleteFile;

            if selected_file.name.starts_with(CONFIG_FILE_NAME) {
                return;
            }
            
            let file_path = if !selected_file.is_folder && selected_file.collection_name.is_some() {
                format!("{}/{}", 
                    selected_file.collection_name.as_ref().unwrap(),
                    selected_file.name)
            } else {
                selected_file.name.clone()
            };
            
            self.file_operation_state = Some(FileOperationState::Delete {
                name: file_path,
                is_folder: selected_file.is_folder,
                scope: selected_file.scope,
            });
        }
        self.close_modal();
    }

    fn handle_delete(&mut self) {
        if let Some(FileOperationState::Delete { name, is_folder, scope }) = &self.file_operation_state {
            match self.fs.delete_file_or_folder(name, *is_folder, *scope) {
                Ok(_) => {
                    self.ui_state.message = format!("{} deleted successfully", 
                        if *is_folder { "Folder" } else { "File" });
                    self.reload_collections();
                },
                Err(e) => {
                    self.ui_state.message = format!("Error deleting {}: {}", 
                        if *is_folder { "folder" } else { "file" }, e);
                }
            }
        }
    }

    fn show_password_prompt(&mut self) {
        self.modal_manager.show_modal(ModalType::Password);
        self.mode = Mode::Password;
    }

    fn show_new_file_modal(&mut self) {
        let folder_context = get_selected_folder_context(self.ui_state.collection_state.selected());
        let parent_folder = folder_context.map(|(folder, _)| folder);
        
        self.modal_manager.show_modal(ModalType::NewFile { parent_folder });
        self.mode = Mode::NewFile;
    }

    fn show_edit_file_modal(&mut self, name: String, is_folder: bool, current_scope: CollectionScope) {
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
    
        if let Some(file) = parse_selected_file(selected) {
            match file {
                SelectedFile::Config(scope) => Some(SelectedFileInfo {
                    name: CONFIG_FILE_NAME.to_string(),
                    collection_name: None,
                    is_folder: false,
                    scope,
                }),
                SelectedFile::Sql { collection, filename, scope } => {
                    let is_folder = filename.is_empty();
                    let name = if is_folder { collection.clone() } else { filename };
                    Some(SelectedFileInfo {
                        name,
                        collection_name: if is_folder { None } else { Some(collection) },
                        is_folder,
                        scope,
                    })
                }
                SelectedFile::Folder { name, scope } => {
                    Some(SelectedFileInfo {
                        name,
                        collection_name: None,
                        is_folder: true,
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
impl App<'_> {
    pub fn process_async_results(&mut self) {
        if let Some(handle) = &mut self.query_state.pending_async_operation {
            if handle.is_finished() {
                let handle = std::mem::take(&mut self.query_state.pending_async_operation).unwrap();
                
                match tokio::task::block_in_place(|| futures::executor::block_on(handle)) {
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