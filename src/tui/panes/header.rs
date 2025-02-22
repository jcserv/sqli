use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::{app::{App, Mode}, navigation::{FocusType, Navigable, PaneId}};
use super::traits::Instructions;

pub struct HeaderPane;

impl HeaderPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect) {
        let focus_type = if let Some(info) = app.navigation.get_pane_info(PaneId::Header) {
            info.focus_type
        } else {
            FocusType::Inactive
        };

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
        // let conn_chunks = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints([
        //         Constraint::Min(20),        // Connection selector
        //         Constraint::Length(10),     // Run button
        //     ])
        //     .split(conn_area);
            
        let focus_style = match focus_type {
            FocusType::Editing => Style::default().fg(Color::LightBlue).bold(),
            FocusType::Active => Style::default().fg(Color::LightBlue),
            FocusType::Inactive => Style::default().fg(Color::White),
        };
        
        let connection_block = Block::default()
            .title("Connection")
            .title_style(focus_style)
            .borders(Borders::ALL)
            .border_style(focus_style);
        
        // let inner_conn_area = connection_block.inner(conn_area);
        frame.render_widget(connection_block, conn_area);
    }
}

impl Instructions for HeaderPane {
    fn get_instructions(&self, _app: &App) -> Line<'static> {
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


impl Navigable for HeaderPane {
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        if app.mode != Mode::Normal || !app.navigation.is_active(PaneId::Header) {
            return Ok(false);
        }
        
        let is_editing = app.is_pane_in_edit_mode(PaneId::Header);
        
        match key_event.code {
            KeyCode::Esc if is_editing => {
                self.deactivate(app)
            },
            KeyCode::Enter | KeyCode::Char(' ') if !is_editing => {
                self.activate(app)
            },
            _ => Ok(false)
        }
    }
    
    fn handle_mouse_event(&self, app: &mut App, _mouse_event: MouseEvent) -> Result<bool> {
        if app.navigation.is_active(PaneId::Header) {
            return self.activate(app)
        }
        app.navigation.activate_pane(PaneId::Header)?;
        Ok(false)
    }
    
    fn activate(&self, app: &mut App) -> Result<bool> {
        app.navigation.start_editing(PaneId::Header)?;
        Ok(false)
    }
    
    fn deactivate(&self, app: &mut App) -> Result<bool> {
        app.navigation.stop_editing(PaneId::Header)?;
        Ok(false)
    }
}