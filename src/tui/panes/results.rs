use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    layout::Rect, prelude::*, style::{Color, Style}, text::Line, widgets::{Block, Borders}, Frame
};

use crate::tui::app::{App, Focus, Mode, Tab};

use super::traits::{Instructions, PaneEventHandler};

pub struct ResultsPane;

impl ResultsPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect) {
        let focus_style = if app.focus == Focus::Result {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title_top("Results").title_alignment(Alignment::Left)
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
        Ok(false)
    }
    
    fn handle_mouse_event(&self, app: &mut App, _mouse_event: MouseEvent) -> Result<bool> {
        app.select_tab(Tab::Result);
        Ok(false)
    }
}