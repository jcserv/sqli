use anyhow::Result;
use crossterm::event::MouseEvent;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use tui_textarea::{TextArea, Input, Key};
use tui_tree_widget::{TreeItem, TreeState};

use super::searchable_textarea::SearchableTextArea;
use super::ui::UI;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
    Search,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Tab {
    Collections,
    Workspace,
    Result,
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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Focus {
    Collections,
    CollectionsEdit,
    Workspace,      
    WorkspaceEdit,  
    Result,         
}

pub struct App<'a> {
    pub mode: Mode,
    pub current_tab: Tab,
    pub focus: Focus,         
    pub workspace: SearchableTextArea<'a>,
    pub command_input: String,
    pub message: String,
    pub queries: Vec<String>,
    pub collection_state: TreeState<String>,
    pub collection_items: Vec<TreeItem<'a, String>>,
    pub should_quit: bool,
    pub search: SearchBox<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let mut workspace = SearchableTextArea::default();
        workspace.init();

        let collection_items = super::collections_pane::CollectionsPane::load_collections();
        Self {
            mode: Mode::Normal,
            current_tab: Tab::Collections,
            focus: Focus::Collections,
            workspace,
            command_input: String::new(),
            message: String::new(),
            queries: Vec::new(),
            collection_state: TreeState::default(),
            collection_items,
            should_quit: false,
            search: SearchBox::default(),
        }
    }

    pub fn tick(&mut self) {
        // Update any app state that needs to change every tick
    }

    pub fn handle_key(&mut self, ui: &UI, key_event: KeyEvent) -> Result<bool> {
        match self.mode {
            Mode::Normal => ui.handle_key_event(self, key_event),
            Mode::Command => self.handle_command_mode(key_event),
            Mode::Search => self.handle_search_mode(key_event),
        }
    }

    pub fn handle_mouse(&mut self, ui: &UI, mouse_event: MouseEvent) -> Result<bool> {
        ui.handle_mouse_event(self, mouse_event)
    }

    fn handle_search_mode(&mut self, key_event: KeyEvent) -> Result<bool> {
        let input = Input::from(key_event);
        match input {
            Input { key: Key::Esc, .. } => {
                self.search.open = false;
                self.mode = Mode::Normal;
                self.workspace.set_search_pattern("")?;
                Ok(false)
            }
            Input { key: Key::Enter, .. } => {
                if self.search.replace_mode {
                    let pattern = self.search.textarea.lines()[0].as_str();
                    let replacement = self.search.textarea.lines().get(1).map(|s| s.as_str()).unwrap_or("");
                    self.workspace.set_search_pattern(pattern)?;
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        let count = self.workspace.replace_all(replacement);
                        self.message = format!("Replaced {} occurrences", count);
                    } else {
                        if self.workspace.replace_next(replacement) {
                            self.message = "Replaced occurrence".to_string();
                        } else {
                            self.message = "No more matches".to_string();
                        }
                    }
                } else {
                    let pattern = self.search.textarea.lines()[0].as_str();
                    self.workspace.set_search_pattern(pattern)?;
                    if !self.workspace.search_forward(true) {
                        self.message = "Pattern not found".to_string();
                    }
                }
                self.search.open = false;
                self.mode = Mode::Normal;
                Ok(false)
            }
            Input { 
                key: Key::Char('n'),
                ctrl: true,
                ..
            } => {
                if !self.workspace.search_forward(false) {
                    self.message = "Pattern not found".to_string();
                }
                Ok(false)
            }
            Input {
                key: Key::Char('p'),
                ctrl: true,
                ..
            } => {
                if !self.workspace.search_back(false) {
                    self.message = "Pattern not found".to_string();
                }
                Ok(false)
            }
            _ => {
                self.search.textarea.input(input);
                if let Some(pattern) = self.search.textarea.lines().first() {
                    self.workspace.set_search_pattern(pattern)?;
                }
                Ok(false)
            }
        }
    }

    fn handle_command_mode(&mut self, key_event: KeyEvent) -> Result<bool> {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                Ok(false)
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command_input.clear();
                Ok(false)
            }
            KeyCode::Enter => {
                self.execute_command()?;
                self.mode = Mode::Normal;
                self.command_input.clear();
                Ok(false)
            }
            KeyCode::Char(c) => {
                self.command_input.push(c);
                Ok(false)
            }
            KeyCode::Backspace => {
                self.command_input.pop();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    pub fn select_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
        self.focus = match tab {
            Tab::Collections => Focus::Collections,
            Tab::Workspace => Focus::Workspace,
            Tab::Result => Focus::Result,
        };
    }

    pub fn cycle_tab(&mut self) {
        match self.current_tab {
            Tab::Collections => self.select_tab(Tab::Workspace),
            Tab::Workspace => self.select_tab(Tab::Result),
            Tab::Result => self.select_tab(Tab::Collections),
        }
    }

    fn execute_command(&mut self) -> Result<()> {
        let cmd = self.command_input.trim();
        match cmd {
            "w" => self.save_query(),
            "q" => self.should_quit = true,
            "wq" => {
                self.save_query();
                self.should_quit = true;
            }
            _ => self.message = format!("Unknown command: {}", cmd),
        }
        Ok(())
    }

    pub fn save_query(&mut self) {
        let content = self.workspace.get_content();
        if !content.is_empty() {
            self.queries.push(content);
            self.message = "Query saved".to_string();
        }
    }
}