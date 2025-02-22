use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use tui_textarea::TextArea;
use tui_tree_widget::{TreeItem, TreeState};

use crate::query::{self, execute_query};
use crate::sql::interface::QueryResult;

use super::navigation::{NavigationManager, PaneId};
use super::ui::UI;
use super::widgets::password_modal::PasswordModal;
use super::widgets::searchable_textarea::SearchableTextArea;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
    Search,
    Password,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    None,
    ExecuteQuery,
    SaveQuery,
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

pub struct App<'a> {
    pub command_input: String,
    pub mode: Mode,
    pub message: String,

    pub pending_command: AppCommand,
    pub pending_async_operation: Option<tokio::task::JoinHandle<AsyncCommandResult>>,

    pub selected_connection: Option<String>,

    pub navigation: NavigationManager,

    pub collection_state: TreeState<String>,
    pub collection_items: Vec<TreeItem<'a, String>>,
    pub workspace: SearchableTextArea<'a>,
    pub search: SearchBox<'a>,
    pub query_result: QueryResult,
    pub password_modal: Option<PasswordModal<'a>>,

    pub should_quit: bool,
}

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

        let connection = Some("nopass".to_string());
        
        Self {
            command_input: String::new(),
            mode: Mode::Normal,
            message: String::new(),
            pending_command: AppCommand::None,
            pending_async_operation: None,
            selected_connection: connection,
            navigation,
            collection_state: TreeState::default(),
            collection_items,
            workspace,
            search: SearchBox::default(),
            query_result: QueryResult::default(), 
            password_modal: None,
            should_quit: false,
        }
    }

    pub fn process_async_results(&mut self) {
        if let Some(handle) = &mut self.pending_async_operation {
            if handle.is_finished() {
                let handle = std::mem::take(&mut self.pending_async_operation).unwrap();
                
                match tokio::task::block_in_place(|| futures::executor::block_on(async {
                    handle.await
                })) {
                    Ok(result) => {
                        match result.command {
                            AppCommand::ExecuteQuery => {
                                let sql = self.workspace.get_content();
                                let connection = self.selected_connection.clone();
                                
                                match tokio::task::block_in_place(|| {
                                    futures::executor::block_on(async {
                                        crate::query::execute_query(
                                            sql,
                                            None,
                                            connection,
                                            None,
                                        ).await
                                    })
                                }) {
                                    Ok(query_result) => {
                                        self.query_result = query_result;
                                        self.message = format!(
                                            "Query executed successfully in {}ms",
                                            self.query_result.execution_time.as_millis()
                                        );
                                    }
                                    Err(e) => {
                                        if e.to_string().contains("password authentication failed") {
                                            self.show_password_prompt();
                                        } else {
                                            self.message = format!("Error executing query: {}", e);
                                            self.query_result = QueryResult::empty();
                                        }
                                    }
                                }
                            },
                            AppCommand::SaveQuery => {
                                self.save_query();
                            },
                            AppCommand::None => {},
                        }
    
                        if let Some(msg) = result.message {
                            self.message = msg;
                        }
                    },
                    Err(e) => {
                        self.message = format!("Error in async operation: {}", e);
                    }
                }
            }
        }
    }

    pub fn tick(&mut self) {
        if self.pending_command != AppCommand::None {
            match self.pending_command {
                AppCommand::ExecuteQuery => {
                    self.check_and_execute_query();
                },
                AppCommand::SaveQuery => {
                    self.save_query();
                },
                AppCommand::None => {},
            }
            
            self.pending_command = AppCommand::None;
        }
    }

    pub fn handle_key(&mut self, ui: &mut UI, key_event: KeyEvent) -> anyhow::Result<bool> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(true);
            }
            (KeyCode::Tab, _) if !self.is_edit_mode() => {
                _ = self.navigation.cycle_pane(false);
                return Ok(false);
            }
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                self.mode = Mode::Command;
                return Ok(false);
            }
            _ => {
                match self.mode {
                    Mode::Password => {
                        match key_event.code {
                            KeyCode::Enter => {
                                self.handle_password_submit();
                                Ok(false)
                            }
                            KeyCode::Esc => {
                                self.close_password_prompt();
                                Ok(false)
                            }
                            _ => {
                                if let Some(modal) = &mut self.password_modal {
                                    modal.textarea.input(tui_textarea::Input::from(key_event));
                                }
                                Ok(false)
                            }
                        }
                    }
                    _ => {
                        return ui.handle_key_event(self, key_event);
                    }
                }
            }
        }
    }

    pub fn handle_mouse(&mut self, ui: &super::ui::UI, mouse_event: MouseEvent) -> anyhow::Result<bool> {
        ui.handle_mouse_event(self, mouse_event)
    }

    pub fn check_and_execute_query(&mut self) {
        if let Some(conn_name) = &self.selected_connection {
            match query::get_connection(conn_name) {
                Ok(Some(conn)) => {
                    if conn.requires_password() {
                        self.show_password_prompt();
                        return;
                    }
                }
                Ok(None) => {
                    self.message = format!("Connection '{}' not found", conn_name);
                    return;
                }
                Err(e) => {
                    self.message = format!("Error checking connection: {}", e);
                    return;
                }
            }
        }

        self.execute_query_with_password(None);
    }

    fn execute_query_with_password(&mut self, password: Option<String>) {
        let sql = self.workspace.get_content();
        let connection = self.selected_connection.clone();
        
        let handle = tokio::spawn(async move {
            match execute_query(sql, None, connection, password).await {
                Ok(_) => AsyncCommandResult::new(AppCommand::ExecuteQuery),
                Err(e) => AsyncCommandResult::with_message(
                    AppCommand::ExecuteQuery,
                    format!("Query error: {}", e)
                ),
            }
        });
        
        self.pending_async_operation = Some(handle);
    }

    pub fn save_query(&mut self) {
        let content = self.workspace.get_content();
        if !content.is_empty() {
            // TODO
        }
    }

    pub fn show_password_prompt(&mut self) {
        self.password_modal = Some(PasswordModal::default());
        self.mode = Mode::Password;
    }

    pub fn handle_password_submit(&mut self) {
        let password = self.password_modal.as_ref()
            .and_then(|modal| modal.textarea.lines().first())
            .map(|line| line.clone());
            
        self.execute_query_with_password(password);
        self.password_modal = None;
        self.mode = Mode::Normal;
    }

    pub fn close_password_prompt(&mut self) {
        self.password_modal = None;
        self.mode = Mode::Normal;
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