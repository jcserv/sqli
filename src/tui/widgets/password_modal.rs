// In src/tui/widgets/password_modal.rs

use ratatui::{
    prelude::*,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

pub struct PasswordModal<'a> {
    pub textarea: TextArea<'a>,
    pub error: Option<String>,
}

impl<'a> Default for PasswordModal<'a> {
    fn default() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Password")
        );
        Self {
            textarea,
            error: None,
        }
    }
}

impl<'a> PasswordModal<'a> {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(60, 12, area);
        
        frame.render_widget(Clear, popup_area);
        
        let block = Block::default()
            .title("Enter Password")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
            
        frame.render_widget(block.clone(), popup_area);
        let inner_area = block.inner(popup_area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Password input
                Constraint::Length(1), // Error message
                Constraint::Length(1), // Gap
                Constraint::Length(3), // Buttons
            ])
            .split(inner_area);
            
        frame.render_widget(&self.textarea, chunks[0]);
        
        if let Some(error) = &self.error {
            let err_msg = format!(
                "Error: {}",
                error,
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    err_msg.red().bold(),
                ])),
                chunks[1],
            );
        }
        
        let button_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[3]);
            
        frame.render_widget(
            Paragraph::new("[ Cancel ]")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Red)),
            button_layout[0],
        );
        
        frame.render_widget(
            Paragraph::new("[ Submit ]")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Green)),
            button_layout[1],
        );
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