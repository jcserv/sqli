use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
    buffer::Buffer,
};

use super::button::{Button, Theme, BLUE};

pub struct DialogButton<'a> {
    pub label: &'a str,
    pub theme: Theme,
    pub callback: Box<dyn Fn() -> anyhow::Result<()>>,
}

impl<'a> DialogButton<'a> {
    pub fn new(label: &'a str, callback: Box<dyn Fn() -> anyhow::Result<()>>) -> Self {
        Self {
            label,
            theme: BLUE,
            callback,
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
}

impl<'a, W> ModalDialog<'a, W> 
where
    W: Widget,
{
    pub fn new(content: DialogContent<'a, W>, width_percent: Option<u16>, height_percent: Option<u16>) -> Self {
        let width_percent = width_percent.unwrap_or(40);
        let height_percent = height_percent.unwrap_or(35);
        Self {
            content,
            width_percent,
            height_percent,
        }
    }

    pub fn with_dimensions(mut self, width_percent: u16, height_percent: u16) -> Self {
        self.width_percent = width_percent;
        self.height_percent = height_percent;
        self
    }
}

impl<'a, W> Widget for ModalDialog<'a, W>
where
    W: Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::default().bg(Color::Black).fg(Color::Gray));
        
        let modal_area = centered_rect(self.width_percent, self.height_percent, area);
        
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
            
        for (i, button) in self.content.buttons.iter().enumerate() {
            Button::new(button.label)
                .theme(button.theme)
                .render(button_layout[i * 2 + 1], buf);
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