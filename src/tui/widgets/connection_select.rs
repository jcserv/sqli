use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    layout::Rect,
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
    // Store positions for event handling
    dropdown_area: Option<Rect>,
    button_area: Option<Rect>,
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
            dropdown_area: None,
            button_area: None,
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

    pub fn render(&mut self, frame: &mut Frame, selector_area: Rect, button_area: Rect) {
        self.dropdown_area = Some(selector_area);
        self.button_area = Some(button_area);
    
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
    
        frame.render_widget(select, selector_area);        
        frame.render_widget(button, button_area);
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Tab => {
                self.cycle_focus();
                return true;
            },
            KeyCode::Enter if key_event.modifiers.contains(KeyModifiers::CONTROL) 
                         || key_event.modifiers.contains(KeyModifiers::SUPER) => {
                self.button_state = ButtonState::Pressed;
                return true;
            },
            KeyCode::Enter if self.focus == SelectorFocus::Button => {
                self.button_state = ButtonState::Pressed;
                return true;
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
                            return true;
                        }
                    },
                    SelectorFocus::Button => {
                        let mut button = self.run_button.clone();
                        if button.handle_key_event(key_event, true) {
                            self.button_state = ButtonState::Pressed;
                            return true;
                        }
                    },
                    SelectorFocus::None => (),
                }
            }
        }
        false
    }

    pub fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> bool {
        if let (Some(selector_area), Some(button_area)) = (self.dropdown_area, self.button_area) {
            match mouse_event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    let mouse_x = mouse_event.column;
                    let mouse_y = mouse_event.row;

                    if mouse_x >= selector_area.x && 
                       mouse_x < selector_area.x + selector_area.width &&
                       mouse_y >= selector_area.y && 
                       mouse_y < selector_area.y + selector_area.height {
                        
                        self.focus = SelectorFocus::Dropdown;
                        
                        let options_ref: Vec<&str> = self.connection_names.iter()
                            .map(|s| s.as_str())
                            .collect();
                        
                        let mut select = Select::new(options_ref);
                        select.set_selected(self.selected_index);
                        
                        if select.handle_mouse_event(mouse_event, selector_area) {
                            self.selected_index = select.selected();
                            return true;
                        }
                    }
                    
                    else if mouse_x >= button_area.x && 
                            mouse_x < button_area.x + button_area.width &&
                            mouse_y >= button_area.y && 
                            mouse_y < button_area.y + button_area.height {
                        
                        self.focus = SelectorFocus::Button;
                        self.button_state = ButtonState::Pressed;
                        return true;
                    }
                },
                MouseEventKind::Up(MouseButton::Left) => {
                    if self.button_state == ButtonState::Pressed {
                        let mouse_x = mouse_event.column;
                        let mouse_y = mouse_event.row;
                        
                        if mouse_x >= button_area.x && 
                           mouse_x < button_area.x + button_area.width &&
                           mouse_y >= button_area.y && 
                           mouse_y < button_area.y + button_area.height {
                            return true;
                        } else {
                            self.button_state = ButtonState::Focused;
                        }
                    }
                },
                _ => {}
            }
        }
        false
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