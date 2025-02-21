use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Widget},
};

/// A dropdown select widget for ratatui
#[derive(Debug, Clone)]
pub struct Select<'a> {
    /// Options to display in the dropdown
    options: Vec<&'a str>,
    /// Currently selected option index
    selected: usize,
    /// Title of the select widget
    title: &'a str,
    /// Whether the dropdown is currently open
    is_open: bool,
    /// Normal style for the select box
    normal_style: Style,
    /// Style for the select box when it has focus
    focused_style: Style,
    /// Style for the dropdown list items
    item_style: Style,
    /// Style for the selected dropdown list item
    selected_style: Style,
}

impl<'a> Select<'a> {
    pub fn new(options: Vec<&'a str>) -> Self {
        Self {
            options,
            selected: 0,
            title: "",
            is_open: false,
            normal_style: Style::default(),
            focused_style: Style::default().fg(Color::LightBlue),
            item_style: Style::default(),
            selected_style: Style::default().fg(Color::Black).bg(Color::LightBlue),
        }
    }

    pub const fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub const fn normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    pub const fn focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    pub const fn item_style(mut self, style: Style) -> Self {
        self.item_style = style;
        self
    }

    pub const fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.options.get(self.selected).copied()
    }

    pub fn set_selected(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected = index;
        }
    }

    pub fn toggle_dropdown(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn close_dropdown(&mut self) {
        self.is_open = false;
    }

    pub fn select_next(&mut self) {
        if !self.options.is_empty() {
            self.selected = (self.selected + 1) % self.options.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.options.is_empty() {
            self.selected = if self.selected > 0 {
                self.selected - 1
            } else {
                self.options.len().saturating_sub(1)
            };
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent, has_focus: bool) -> bool {
        if !has_focus {
            return false;
        }

        match key_event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_dropdown();
                true
            }
            KeyCode::Esc => {
                if self.is_open {
                    self.close_dropdown();
                    true
                } else {
                    false
                }
            }
            KeyCode::Down | KeyCode::Char('j') if self.is_open => {
                self.select_next();
                true
            }
            KeyCode::Up | KeyCode::Char('k') if self.is_open => {
                self.select_prev();
                true
            }
            _ => false,
        }
    }

    pub fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> bool {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let position = Position::new(mouse_event.column, mouse_event.row);
                
                if area.contains(position) {
                    if !self.is_open {
                        self.toggle_dropdown();
                        return true;
                    } else {
                        let dropdown_height = self.options.len().min(5) as u16;
                        let dropdown_area = Rect::new(
                            area.x,
                            area.y + 1,
                            area.width,
                            dropdown_height,
                        );

                        if dropdown_area.contains(position) {
                            let clicked_index = (position.y - dropdown_area.y) as usize;
                            if clicked_index < self.options.len() {
                                self.selected = clicked_index;
                                self.close_dropdown();
                                return true;
                            }
                        } else if area.contains(position) {
                            self.close_dropdown();
                            return true;
                        }
                    }
                } else if self.is_open {
                    self.close_dropdown();
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

impl<'a> Widget for Select<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.is_open {
            self.focused_style
        } else {
            self.normal_style
        };
        
        let inner_area = area;
        
        let selected_text = self.options.get(self.selected).copied().unwrap_or("");
        
        let dropdown_indicator = if self.is_open { " ▲" } else { " ▼" };
        let title_text = if self.title.is_empty() { 
            "".to_string() 
        } else { 
            format!("{}:", self.title) 
        };
        
        let text = Line::from(vec![
            Span::styled(title_text, style),
            Span::styled(selected_text, style),
            Span::styled(dropdown_indicator, style),
        ]);
        
        buf.set_style(inner_area, style);
        buf.set_line(
            inner_area.x,
            inner_area.y,
            &text,
            inner_area.width,
        );

        if self.is_open {
            let dropdown_height = self.options.len().min(5) as u16;
            let dropdown_area = Rect::new(
                area.x,
                area.y + 1,
                area.width,
                dropdown_height,
            );
            
            Clear.render(dropdown_area, buf);
            
            let dropdown_block = Block::default()
                .borders(Borders::ALL)
                .border_style(self.normal_style);
            
            let dropdown_block_clone = dropdown_block.clone();
            dropdown_block.render(dropdown_area, buf);
            
            let inner_dropdown = dropdown_block_clone.inner(dropdown_area);
            
            for (i, option) in self.options.iter().enumerate().take(5) {
                let option_style = if i == self.selected {
                    self.selected_style
                } else {
                    self.item_style
                };
                
                let text = Line::from(*option);
                buf.set_style(
                    Rect::new(inner_dropdown.x, inner_dropdown.y + i as u16, inner_dropdown.width, 1),
                    option_style,
                );
                buf.set_line(
                    inner_dropdown.x,
                    inner_dropdown.y + i as u16,
                    &text,
                    inner_dropdown.width,
                );
            }
        }
    }
}