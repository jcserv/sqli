use std::any::Any;

use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
    buffer::Buffer,
    Frame,
};

use super::button::{Button, Theme, BLUE};

pub trait ModalHandler: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModalAction>;    
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<ModalAction>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

#[derive(Debug, PartialEq)]
pub enum ModalAction {
    /// No action needed
    None,
    /// Close the modal
    Close,
    /// Custom action with a string identifier
    Custom(String),
}

pub struct DialogButton<'a> {
    pub label: &'a str,
    pub theme: Theme,
    pub action: String,
    pub rect: Option<Rect>,
}

impl<'a> DialogButton<'a> {
    pub fn new(label: &'a str, action: impl Into<String>) -> Self {
        Self {
            label,
            theme: BLUE,
            action: action.into(),
            rect: None,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

pub struct DialogContent<'a, W> {
    pub title: &'a str,
    pub content_widget: W,
    pub buttons: Vec<DialogButton<'a>>,
}

pub struct ModalDialog<'a, W> {
    content: DialogContent<'a, W>,
    width_percent: u16,
    height_percent: u16,
    modal_area: Option<Rect>,
}

impl<'a, W> ModalDialog<'a, W> 
where
    W: Widget,
{
    pub fn new(content: DialogContent<'a, W>) -> Self {
        Self {
            content,
            width_percent: 40,
            height_percent: 35,
            modal_area: None,
        }
    }

    pub fn with_dimensions(mut self, width_percent: u16, height_percent: u16) -> Self {
        self.width_percent = width_percent;
        self.height_percent = height_percent;
        self
    }

    pub fn handle_mouse_event(&self, mouse_event: MouseEvent, _area: Rect) -> Result<ModalAction> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let position = Position::new(mouse_event.column, mouse_event.row);
                
                if let Some(modal_area) = self.modal_area {
                    if !modal_area.contains(position) {
                        return Ok(ModalAction::Close);
                    }
                }
                
                for button in &self.content.buttons {
                    if let Some(button_rect) = button.rect {
                        if button_rect.contains(position) {
                            return Ok(ModalAction::Custom(button.action.clone()));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(ModalAction::None)
    }

    pub fn handle_key_event(&self, key_event: KeyEvent) -> Result<ModalAction> {
        use crossterm::event::KeyCode;
        
        match key_event.code {
            KeyCode::Esc => Ok(ModalAction::Close),
            _ => Ok(ModalAction::None),
        }
    }
}

impl<'a, W> Widget for ModalDialog<'a, W>
where
    W: Widget,
{
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::default().bg(Color::Black).fg(Color::Gray));
        
        let modal_area = centered_rect(self.width_percent, self.height_percent, area);
        self.modal_area = Some(modal_area);
        
        let block = Block::default()
            .title(self.content.title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .style(Style::default().bg(Color::DarkGray));
            
        let inner_area = block.inner(modal_area);        
        block.render(modal_area, buf);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(3),    // Content area
                Constraint::Length(3), // Buttons
            ])
            .split(inner_area);
            
        self.content.content_widget.render(chunks[0], buf);
        
        let button_constraints: Vec<Constraint> = std::iter::once(Constraint::Percentage((100 - self.content.buttons.len() as u16 * 20) / 2))
            .chain(self.content.buttons.iter().flat_map(|_| {
                vec![
                    Constraint::Length(12),  // Button width
                    Constraint::Length(2),   // Gap between buttons
                ]
            }))
            .chain(std::iter::once(Constraint::Percentage((100 - self.content.buttons.len() as u16 * 20) / 2)))
            .collect();
        
        let button_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(button_constraints)
            .split(chunks[1]);
            
        for (i, button) in self.content.buttons.iter_mut().enumerate() {
            let button_area = button_layout[i * 2 + 1];
            button.rect = Some(button_area);
            Button::new(button.label)
                .theme(button.theme)
                .render(button_area, buf);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}