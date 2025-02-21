use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

use super::app::{App, Focus};

pub struct ResultsPane;

impl ResultsPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect) {
        let focus_style = if app.focus == Focus::Result {
            Style::default().fg(Color::LightBlue)// .bold()
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title("Results")
            .title_style(focus_style)
            .borders(Borders::ALL)
            .border_style(focus_style);

        frame.render_widget(block, area);
        
        // TODO: Render query results here when implemented
    }
}