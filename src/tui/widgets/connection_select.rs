use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
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
    focus_style: Style,
    is_open: bool,
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
            focus_style: Style::default(),
            is_open: false,
        }
    }

    pub fn set_focus_style(&mut self, style: Style) {
        self.focus_style = style;
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
    
        let dropdown_display = {
            let selected_text = self.connection_names.get(self.selected_index)
                .cloned()
                .unwrap_or_default();
            
            let dropdown_indicator = if self.focus == SelectorFocus::Dropdown { " ▼" } else { " ▼" };
            
            Line::from(vec![
                Span::raw(selected_text),
                Span::styled(dropdown_indicator, self.focus_style),
            ])
        };
        
        frame.render_widget(Paragraph::new(dropdown_display), selector_area);        
        if self.focus == SelectorFocus::Dropdown && self.is_open {
            let dropdown_height = self.connection_names.len().min(5) as u16;
            let dropdown_area = Rect::new(
                selector_area.x,
                selector_area.y + 1,
                selector_area.width,
                dropdown_height,
            );
            
            Clear.render(dropdown_area, frame.buffer_mut());
            
            let dropdown_block = Block::default()
                .borders(Borders::ALL)
                .border_style(self.focus_style);            
            let dropdown_block_inner = dropdown_block.clone();
            dropdown_block.render(dropdown_area, frame.buffer_mut());
            let inner_dropdown = dropdown_block_inner.inner(dropdown_area);
        
            for (i, option) in self.connection_names.iter().enumerate().take(5) {
                let option_style = if i == self.selected_index {
                    Style::default().fg(Color::Black).bg(Color::LightBlue)
                } else {
                    Style::default()
                };
                
                let text = Line::from(option.as_str());
                frame.buffer_mut().set_style(
                    Rect::new(inner_dropdown.x, inner_dropdown.y + i as u16, inner_dropdown.width, 1),
                    option_style,
                );
                frame.buffer_mut().set_line(
                    inner_dropdown.x,
                    inner_dropdown.y + i as u16,
                    &text,
                    inner_dropdown.width,
                );
            }
        }
        
        let button = Button::new("Run")
            .normal_style(Style::default())
            .focused_style(self.focus_style)
            .pressed_style(Style::default().fg(Color::Black).bg(Color::LightBlue))
            .state(self.button_state);
        
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
            KeyCode::Enter if self.focus == SelectorFocus::Dropdown => {
                self.toggle_dropdown();
                return true;
            },
            _ => {
                match self.focus {
                    SelectorFocus::Dropdown => {
                        match key_event.code {
                            KeyCode::Down | KeyCode::Char('j') => {
                                if !self.connection_names.is_empty() {
                                    self.selected_index = (self.selected_index + 1) % self.connection_names.len();
                                }
                                return true;
                            },
                            KeyCode::Up | KeyCode::Char('k') => {
                                if !self.connection_names.is_empty() {
                                    self.selected_index = if self.selected_index > 0 {
                                        self.selected_index - 1
                                    } else {
                                        self.connection_names.len() - 1
                                    };
                                }
                                return true;
                            },
                            _ => {},
                        }
                    },
                    SelectorFocus::Button => {
                        match key_event.code {
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                self.button_state = ButtonState::Pressed;
                                return true;
                            },
                            _ => {},
                        }
                    },
                    SelectorFocus::None => {},
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
                        self.toggle_dropdown();
                        return true;
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
    
    fn toggle_dropdown(&mut self) {
        self.is_open = !self.is_open;
    }
}