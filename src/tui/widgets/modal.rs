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

use super::button::{Button, Theme};

#[derive(Debug, Clone)]
pub struct DialogButton<'a> {
    pub button: Button<'a>,
    pub action: String,
}

impl<'a> DialogButton<'a> {
    pub fn new(label: &'a str, action: impl Into<String>) -> Self {
        Self {
            button: Button::new(label),
            action: action.into(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.button = self.button.theme(theme);
        self
    }

    pub fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> bool {
        self.button.handle_mouse_event(mouse_event)
    }

    pub fn set_area(&mut self, area: Rect) {
        self.button.set_area(area);
    }
}

impl<'a> Widget for DialogButton<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.button.render(area, buf)
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
    buttons: Vec<DialogButton<'a>>,
}

impl<'a, W> ModalDialog<'a, W> 
where
    W: Widget,
{
    pub fn new(content: DialogContent<'a, W>) -> Self {
        Self {
            buttons: content.buttons,
            content: DialogContent {
                title: content.title,
                content_widget: content.content_widget,
                buttons: Vec::new(),
            },
            width_percent: 40,
            height_percent: 35,
        }
    }

    pub fn with_dimensions(mut self, width_percent: u16, height_percent: u16) -> Self {
        self.width_percent = width_percent;
        self.height_percent = height_percent;
        self
    }

    pub fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<ModalAction> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let position = Position::new(mouse_event.column, mouse_event.row);
                let (modal_area, button_areas) = self.get_layout(area);
                
                if !modal_area.contains(position) {
                    
                    return Ok(ModalAction::Close);
                }
                
                for (i, button) in self.buttons.iter_mut().enumerate() {
                    let (button_rect, _) = &button_areas[i];
                    button.set_area(*button_rect);
                    if button_rect.contains(position) {
                        return Ok(ModalAction::Custom(button.action.clone()));
                    }
                }
            }
            MouseEventKind::Moved => {
                let (_, button_areas) = self.get_layout(area);
                for (button, (button_rect, _)) in self.buttons.iter_mut().zip(button_areas.iter()) {
                    button.set_area(*button_rect);
                    button.handle_mouse_event(mouse_event);
                }
            }
            MouseEventKind::Up(_) => {
                for button in &mut self.buttons {
                    button.handle_mouse_event(mouse_event);
                }
            }
            _ => {}
        }
        Ok(ModalAction::None)
    }

    fn get_layout(&self, area: Rect) -> (Rect, Vec<(Rect, String)>) {
        let modal_area = centered_rect(self.width_percent, self.height_percent, area);
        
        let inner_area = Block::default()
            .title(self.content.title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .inner(modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(3),    // Content area
                Constraint::Length(3), // Buttons
            ])
            .split(inner_area);

        let button_width = 12;
        let gap_width = 2;
        let total_button_width = (button_width + gap_width) * self.buttons.len() as u16;
        
        let left_margin = (chunks[1].width.saturating_sub(total_button_width)) / 2;
        let mut button_areas = Vec::new();
        let mut current_x = chunks[1].x + left_margin;

        for button in &self.buttons {
            let button_area = Rect::new(
                current_x,
                chunks[1].y,
                button_width,
                3
            );
            button_areas.push((button_area, button.action.clone()));
            current_x += button_width + gap_width;
        }

        (modal_area, button_areas)
    }
}

impl<'a, W> Widget for ModalDialog<'a, W>
where
    W: Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::default().bg(Color::Black).fg(Color::Gray));
        
        let (modal_area, _button_areas) = self.get_layout(area);
        
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
        
        let button_constraints: Vec<Constraint> = std::iter::once(Constraint::Percentage((100 - self.buttons.len() as u16 * 20) / 2))
            .chain(self.buttons.iter().flat_map(|_| {
                vec![
                    Constraint::Length(12),  // Button width
                    Constraint::Length(2),   // Gap between buttons
                ]
            }))
            .chain(std::iter::once(Constraint::Percentage((100 - self.buttons.len() as u16 * 20) / 2)))
            .collect();

        let button_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(button_constraints)
            .split(chunks[1]);
            
        for (i, button) in self.buttons.into_iter().enumerate() {
            let button_area = button_layout[i * 2 + 1];
            button.render(button_area, buf);
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ModalAction {
    None,
    Close,
    Custom(String),
}

pub trait ModalHandler: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModalAction>;    
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<ModalAction>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
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