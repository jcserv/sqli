use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
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
        textarea.set_style(Style::default().bg(Color::Black));
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightBlue))
                .title("Password")
        );
        textarea.set_mask_char('*');
        Self {
            textarea,
            error: None,
        }
    }
}

impl<'a> PasswordModal<'a> {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black).fg(Color::Gray)),
            area,
        );
        
        let modal_area = centered_rect(40, 35, area);
        
        let modal_block = Block::default()
            .title("Enter Password")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .style(Style::default().bg(Color::DarkGray));
            
        frame.render_widget(modal_block.clone(), modal_area);
        
        let inner_area = modal_block.inner(modal_area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Password input
                Constraint::Length(3), // Buttons
            ])
            .split(inner_area);
            
        frame.render_widget(&self.textarea, chunks[0]);
        
        let button_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Length(10),
                Constraint::Length(2),
                Constraint::Length(10),
                Constraint::Percentage(35),
            ])
            .split(chunks[1]);
            
        frame.render_widget(
            Block::default()
                .title("Cancel")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
            button_layout[1],
        );
        
        frame.render_widget(
            Block::default()
                .title("Submit")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
            button_layout[3],
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