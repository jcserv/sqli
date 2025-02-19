use std::collections::HashMap;
use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Collections,
    Workspace,
    Result,
}

pub struct App {
    pub mode: Mode,
    pub current_tab: Tab,
    pub query: String,
    pub command_input: String,
    pub message: String,
    pub queries: Vec<String>,
    pub collections: HashMap<String, Vec<String>>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            current_tab: Tab::Collections,
            query: String::new(),
            command_input: String::new(),
            message: String::new(),
            queries: Vec::new(),
            collections: HashMap::new(),
            should_quit: false,
        }
    }

    pub fn tick(&mut self) {
        // Update any app state that needs to change every tick
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) -> Result<bool> {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key_event),
            Mode::Command => self.handle_command_mode(key_event),
        }
    }

    fn handle_normal_mode(&mut self, key_event: KeyEvent) -> Result<bool> {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.should_quit = true;
                Ok(true)
            }
            KeyCode::Char('p') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.mode = Mode::Command;
                Ok(false)
            }
            KeyCode::Tab => {
                self.cycle_tab();
                Ok(false)
            }
            _ => {
                self.handle_query_write(key_event);
                Ok(false)
            },
        }
    }

    fn handle_command_mode(&mut self, key_event: KeyEvent) -> Result<bool> {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
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
        // Parse and execute command
        let cmd = self.command_input.trim();
        match cmd {
            "w" => self.save_query(),
            "q" => self.should_quit = true,
            _ => self.message = format!("Unknown command: {}", cmd),
        }
        Ok(())
    }

    fn handle_query_write(&mut self, key_event: KeyEvent) {
        self.query.push_str(&key_event.code.to_string());
    }

    fn save_query(&mut self) {
        if !self.query.is_empty() {
            self.queries.push(self.query.clone());
            self.message = "Query saved".to_string();
        }
    }
}
