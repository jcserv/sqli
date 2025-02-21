use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    prelude::*,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::{App, Focus};
use super::traits::{Instructions, PaneEventHandler};

pub struct HeaderPane;

impl HeaderPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect) {
        // let focus_style = if app.focus == Focus::Header {
        //     Style::default().fg(Color::LightBlue)
        // } else {
        //     Style::default().fg(Color::White)
        // };

        let app_info_line = Line::from(vec![
            " sqli ".white().bold(),
            "v0.1.0 ".white().into(),
        ]);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12),         // App info
                Constraint::Percentage(70),     // Connection selector
                Constraint::Percentage(30),     // Run button
            ])
            .split(area);

        let title = Paragraph::new(app_info_line)
            .style(Style::default());

        frame.render_widget(title, chunks[0]);
        app.connection_selector.render(frame, chunks[1], chunks[2]);
    }
}

impl Instructions for HeaderPane {
    fn get_instructions(&self, _app: &App) -> Line<'static> {
        Line::from("")
    }
}

impl PaneEventHandler for HeaderPane {
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        if app.connection_selector.handle_key_event(key_event) {
            if app.connection_selector.is_query_triggered() {
                app.execute_query();
            }
            return Ok(false);
        }
        Ok(false)
    }
    
    fn handle_mouse_event(&self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        if app.connection_selector.handle_mouse_event(mouse_event) {
            if app.connection_selector.is_query_triggered() {
                app.execute_query();
            }
            return Ok(false);
        }
        Ok(false)
    }
}