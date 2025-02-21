use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    Frame,
};

use crate::config::ConfigManager;
use crate::tui::widgets::select::Select;
use crate::tui::widgets::button::{Button, ButtonState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorFocus {
    None,
    Dropdown,
    Button,
}

pub struct ConnectionSelector<'a> {
    pub select: Select<'a>,
    pub run_button: Button<'a>,
    pub focus: SelectorFocus,
    connection_names: Vec<String>,
    selected_index: usize,
    button_state: ButtonState,
}

impl<'a> ConnectionSelector<'a> {
    pub fn new() -> Self {
        let connection_names = Self::load_connection_names();
        
        Self {
            select: Select::new(Vec::new())
                .title("Connection")
                .normal_style(Style::default())
                .focused_style(Style::default().fg(Color::LightBlue)),
            run_button: Button::new("Run Query")
                .normal_style(Style::default())
                .focused_style(Style::default().fg(Color::LightBlue))
                .pressed_style(Style::default().fg(Color::Black).bg(Color::LightBlue))
                .state(ButtonState::Normal),
            focus: SelectorFocus::None,
            connection_names,
            selected_index: 0,
            button_state: ButtonState::Normal,
        }
    }

    fn load_connection_names() -> Vec<String> {
        match ConfigManager::new() {
            Ok(config_manager) => {
                match config_manager.list_connections() {
                    Ok(connections) => connections,
                    Err(_) => Vec::new(),
                }
            },
            Err(_) => Vec::new(),
        }
    }

    pub fn render_with_refs(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ])
            .split(area);
    
        let options_ref: Vec<&str> = self.connection_names.iter()
            .map(|s| s.as_str())
            .collect();
    
        let select = Select::new(options_ref)
            .title("Connection")
            .normal_style(Style::default())
            .focused_style(Style::default().fg(Color::LightBlue));
    
        let button = Button::new("Run Query")
            .normal_style(Style::default())
            .focused_style(Style::default().fg(Color::LightBlue))
            .pressed_style(Style::default().fg(Color::Black).bg(Color::LightBlue))
            .state(self.button_state);
    
        frame.render_widget(select, chunks[0]);        
        frame.render_widget(button, chunks[1]);
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool> {
        match key_event.code {
            KeyCode::Tab => {
                self.cycle_focus();
                return Ok(true);
            },
            KeyCode::Enter if key_event.modifiers.contains(KeyModifiers::CONTROL) 
                         || key_event.modifiers.contains(KeyModifiers::SUPER) => {
                self.button_state = ButtonState::Pressed;
                return Ok(true);
            },
            _ => {
                match self.focus {
                    SelectorFocus::Dropdown => {
                        let options_ref: Vec<&str> = self.connection_names.iter()
                            .map(|s| s.as_str())
                            .collect();
                            
                        let mut select = Select::new(options_ref)
                            .title("Connection")
                            .normal_style(Style::default())
                            .focused_style(Style::default().fg(Color::LightBlue));
                            
                        select.set_selected(self.selected_index);
                        
                        if select.handle_key_event(key_event, true) {
                            self.selected_index = select.selected();
                            return Ok(true);
                        }
                    },
                    SelectorFocus::Button => {
                        let mut button = self.run_button.clone();
                        if button.handle_key_event(key_event, true) {
                            self.button_state = ButtonState::Pressed;
                            return Ok(true);
                        }
                    },
                    SelectorFocus::None => (),
                }
            }
        }
        Ok(false)
    }

    pub fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<bool> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ])
            .split(area);

        // First check the select widget
        // Create temporary vector of references for the select widget
        let options_ref: Vec<&str> = self.connection_names.iter()
            .map(|s| s.as_str())
            .collect();
            
        let mut select = Select::new(options_ref)
            .title("Connection")
            .normal_style(Style::default())
            .focused_style(Style::default().fg(Color::LightBlue));
            
        // Set the selected index to match our stored value
        select.set_selected(self.selected_index);
        
        if select.handle_mouse_event(mouse_event, chunks[0]) {
            // Update our selected index from the widget
            self.selected_index = select.selected();
            self.focus = SelectorFocus::Dropdown;
            return Ok(true);
        }

        let mut button = self.run_button.clone();
        if button.handle_mouse_event(mouse_event, chunks[1]) {
            self.button_state = if button.is_pressed() {
                ButtonState::Pressed
            } else {
                ButtonState::Focused
            };
            self.focus = SelectorFocus::Button;
            return Ok(true);
        }

        Ok(false)
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            SelectorFocus::None => SelectorFocus::Dropdown,
            SelectorFocus::Dropdown => SelectorFocus::Button,
            SelectorFocus::Button => SelectorFocus::Dropdown,
        };
        
        self.button_state = match self.focus {
            SelectorFocus::Button => ButtonState::Focused,
            _ => ButtonState::Normal,
        };
    }

    pub fn get_selected_connection(&self) -> Option<&str> {
        self.connection_names.get(self.selected_index).map(|s| s.as_str())
    }

    pub fn is_query_triggered(&mut self) -> bool {
        let triggered = self.button_state == ButtonState::Pressed;
        if triggered {
            self.button_state = ButtonState::Normal;
        }
        triggered
    }
    
    pub fn update_connections(&mut self, connections: Vec<String>) {
        self.connection_names = connections;
        self.selected_index = 0; // Reset selection when connections change
    }
}