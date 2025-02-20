use std::collections::HashMap;
use anyhow::Result;
use crossterm::event::MouseEvent;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use tui_textarea::{TextArea, Input, Key};
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

use super::searchable_textarea::SearchableTextArea;

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
    pub collections: HashMap<String, Vec<String>>,
    pub should_quit: bool,
    pub search: SearchBox<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let mut workspace = SearchableTextArea::default();
        workspace.init();
        
        Self {
            mode: Mode::Normal,
            current_tab: Tab::Collections,
            focus: Focus::Collections,
            workspace,
            command_input: String::new(),
            message: String::new(),
            queries: Vec::new(),
            collections: HashMap::from([
                ("bookDb".to_string(), vec!["getBooks.sql".to_string(), "getBook.sql".to_string()]),
            ]),
            should_quit: false,
            search: SearchBox::default(),
        }
    }

    pub fn tick(&mut self) {
        // Update any app state that needs to change every tick
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) -> Result<bool> {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key_event),
            Mode::Command => self.handle_command_mode(key_event),
            Mode::Search => self.handle_search_mode(key_event),
        }
    }

    pub fn handle_mouse(&mut self, mouse_event: MouseEvent) -> Result<bool> {
        use crossterm::event::{MouseEventKind, MouseButton};
        
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Get terminal size to calculate regions
                let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
                let width = terminal_size.0 as usize;
                let height = terminal_size.1 as usize;
                
                let x = mouse_event.column as usize;
                let y = mouse_event.row as usize;
                
                if y > 1 && y < height - 3 { // Main content area
                    if x < width / 10 {
                        // Left panel - Collections
                        self.select_tab(Tab::Collections);
                    } else {
                        let content_height = height - 5;
                        if y < content_height * 7 / 10 + 2 {
                            // Top-right - Workspace
                            self.select_tab(Tab::Workspace);
                            
                            if self.focus == Focus::Workspace {
                                self.focus = Focus::WorkspaceEdit;
                            }
                        } else {
                            // Bottom-right - Results
                            self.select_tab(Tab::Result);
                        }
                    }
                }
                
                Ok(false)
            },
            _ => Ok(false),
        }
    }

    fn handle_normal_mode(&mut self, key_event: KeyEvent) -> Result<bool> {
        // Global key bindings
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(true);
            }
            (KeyCode::Tab, _) => {
                if self.focus != Focus::WorkspaceEdit {
                    self.cycle_tab();
                    return Ok(false);
                }
                // Forward tab to workspace
                let input = Input::from(key_event);
                self.workspace.input(input);
                return Ok(false);
            }
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                self.mode = Mode::Command;
                return Ok(false);
            }
            _ => {}
        }
    
        // Then handle focus-specific key bindings
        match self.focus {
            Focus::Collections => {
                // Handle collection-specific keys here
                Ok(false)
            }
            Focus::Workspace => {
                match key_event.code {
                    // Press Space to enter edit mode when Workspace is focused
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        self.focus = Focus::WorkspaceEdit;
                        Ok(false)
                    }
                    KeyCode::Char('f') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.mode = Mode::Search;
                        self.search.open = true;
                        self.search.replace_mode = false;
                        self.search.textarea.move_cursor(tui_textarea::CursorMove::End);
                        self.search.textarea.delete_line_by_head();
                        Ok(false)
                    }
                    KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.mode = Mode::Search;
                        self.search.open = true;
                        self.search.replace_mode = true;
                        self.search.textarea.move_cursor(tui_textarea::CursorMove::End);
                        self.search.textarea.delete_line_by_head();
                        Ok(false)
                    }
                    _ => Ok(false)
                }
            }
            Focus::WorkspaceEdit => {
                match key_event.code {
                    KeyCode::Esc => {
                        // Exit edit mode with Escape
                        self.focus = Focus::Workspace;
                        Ok(false)
                    }
                    _ => {
                        // Forward all other keys to the text area
                        let input = Input::from(key_event);
                        self.workspace.input(input);
                        Ok(false)
                    }
                }
            }
            Focus::Result => {
                // Handle result-specific keys here
                Ok(false)
            }
        }
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
                        // Replace all
                        let count = self.workspace.replace_all(replacement);
                        self.message = format!("Replaced {} occurrences", count);
                    } else {
                        // Replace next
                        if self.workspace.replace_next(replacement) {
                            self.message = "Replaced occurrence".to_string();
                        } else {
                            self.message = "No more matches".to_string();
                        }
                    }
                } else {
                    // Search functionality
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
                Ok(true)
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

    fn select_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
        self.focus = match tab {
            Tab::Collections => Focus::Collections,
            Tab::Workspace => Focus::Workspace,
            Tab::Result => Focus::Result,
        };
    }

    fn cycle_tab(&mut self) {
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

    pub fn update_dimensions(&mut self, height: u16) {
        self.workspace.update_dimensions(height);
    }
}