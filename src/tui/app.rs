use std::collections::HashMap;
use anyhow::Result;
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub struct App<'a> {
    pub mode: Mode,
    pub current_tab: Tab,
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
            workspace,
            command_input: String::new(),
            message: String::new(),
            queries: Vec::new(),
            collections: HashMap::new(),
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

    fn handle_normal_mode(&mut self, key_event: KeyEvent) -> Result<bool> {
        if self.current_tab == Tab::Workspace {
            match (key_event.code, key_event.modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                    return Ok(true);
                }
                (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                    self.mode = Mode::Search;
                    self.search.open = true;
                    self.search.replace_mode = false;
                    self.search.textarea.move_cursor(tui_textarea::CursorMove::End);
                    self.search.textarea.delete_line_by_head();
                    return Ok(false);
                }
                (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                    self.mode = Mode::Search;
                    self.search.open = true;
                    self.search.replace_mode = true;
                    self.search.textarea.move_cursor(tui_textarea::CursorMove::End);
                    self.search.textarea.delete_line_by_head();
                    return Ok(false);
                }
                (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                    self.mode = Mode::Command;
                    return Ok(false);
                }
                (KeyCode::Tab, _) => {
                    self.cycle_tab();
                    return Ok(false);
                }
                _ => {
                    let input = Input::from(key_event);
                    self.workspace.input(input);
                    return Ok(false);
                }
            }
        }

        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                Ok(true)
            }
            KeyCode::Tab => {
                self.cycle_tab();
                Ok(false)
            }
            _ => Ok(false),
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

    fn cycle_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Collections => Tab::Workspace,
            Tab::Workspace => Tab::Result,
            Tab::Result => Tab::Collections,
        };
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