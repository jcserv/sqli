use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),          // App title bar
                Constraint::Min(1),             // Connection area
            ])
            .split(area);
        
        let app_info_line = Line::from(vec![
            " sqli ".white().bold(),
            "v0.1.0".dark_gray().into(),
        ]);
        
        let title = Paragraph::new(app_info_line)
            .style(Style::default());
        frame.render_widget(title, chunks[0]);
        
        let conn_area = chunks[1];
        
        let conn_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(20),        // Connection selector
                Constraint::Length(10),     // Run button
            ])
            .split(conn_area);
            
        let focus_style = if app.focus == Focus::Header {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default().fg(Color::White)
        };
        
        let connection_block = Block::default()
            .title("Connections")
            .title_style(focus_style)
            .borders(Borders::ALL)
            .border_style(focus_style);
        
        let inner_conn_area = connection_block.inner(conn_area);
        frame.render_widget(connection_block, conn_area);
        
        app.connection_selector.set_focus_style(focus_style);        
        app.connection_selector.render(frame, inner_conn_area, conn_chunks[1]);
    }
}

impl Instructions for HeaderPane {
    fn get_instructions(&self, app: &App) -> Line<'static> {
        if app.focus == Focus::Header {
            Line::from(vec![
                " Tab ".blue().bold(),
                "Switch Panel ".white().into(),
                " Enter ".blue().bold(),
                "Select/Run ".white().into(),
                " ^P ".blue().bold(),
                "Command ".white().into(),
                " ^C ".blue().bold(),
                "Quit ".white().into(),
            ])
        } else {
            Line::from("")
        }
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