use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    prelude::*,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
    text::Line,
    Frame,
};

use super::{
    app::{App, Focus, Mode},
    traits::{Instructions, PaneEventHandler}
};

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

impl Instructions for ResultsPane {
    fn get_instructions(&self, app: &App) -> Line<'static> {
        if app.focus != Focus::Result || app.mode != Mode::Normal {
            return Line::from("");
        }
        
        Line::from(vec![
            " Tab ".blue().bold(),
            "Switch Panel ".white().into(),
            " Space ".blue().bold(),
            "Select ".white().into(),
            " ^P ".blue().bold(),
            "Command ".white().into(),
            " ^C ".blue().bold(),
            "Quit ".white().into(),
        ])
    }
}

impl PaneEventHandler for ResultsPane {
    fn handle_key_event(&self, _app: &mut App, _key_event: KeyEvent) -> Result<bool> {
        // TODO
        Ok(false)
    }
    
    fn handle_mouse_event(&self, _app: &mut App, _mouse_event: MouseEvent) -> Result<bool> {
        // TODO
        Ok(false)
    }
}