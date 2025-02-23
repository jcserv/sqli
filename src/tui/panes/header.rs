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

use crate::tui::{app::{App, AppCommand, Mode}, navigation::{FocusType, Navigable, PaneId}, widgets::button::{Button, BLUE}};
use super::traits::Instructions;

pub struct HeaderPane;

impl HeaderPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
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
        
        frame.render_widget(&connection_block, conn_area);
        let connection_name = app.selected_connection.clone().unwrap_or_else(|| "No connection selected".to_string());
        frame.render_widget(Span::raw(connection_name), connection_block.inner(conn_area));
        self.render_connection_button(frame, connection_block.inner(conn_area));
    }

    fn render_connection_button(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1), 
                Constraint::Length(15),
                Constraint::Length(1),
            ])
            .split(area);
    
        frame.render_widget(
            Button::new("Run Query").theme(BLUE),
            horizontal[1]
        );
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
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        if app.mode != Mode::Normal || !app.navigation.is_active(PaneId::Header) {
            return Ok(false);
        }

        let info = app.navigation.get_pane_info(PaneId::Header).unwrap();
        match info.focus_type {
            FocusType::Active => {
                match key_event.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.activate(app)
                    },
                    KeyCode::Down => {
                        app.navigation.activate_pane(PaneId::Collections)?;
                        Ok(false)
                    },
                    _ => Ok(false)
                }
            },
            FocusType::Editing => {
                match key_event.code {
                    KeyCode::Esc => {
                        self.deactivate(app)
                    },
                    _ => Ok(false)
                }
            },
            _ => Ok(false)
        }
    }
    
    fn handle_mouse_event(&self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        use crossterm::event::{MouseEventKind, MouseButton};
    
        if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
            if mouse_event.row >= 1 && mouse_event.row <= 3 {
                let terminal_width = crossterm::terminal::size().unwrap_or((80, 24)).0;
    
                let button_width = 15;
                let button_x_start = terminal_width.saturating_sub(button_width + 1);
                let button_x_end = terminal_width.saturating_sub(2);
    
                if mouse_event.column >= button_x_start && mouse_event.column <= button_x_end {
                    app.pending_command = AppCommand::ExecuteQuery;
                    return Ok(false);
                }
            }
        }
    
        if app.navigation.is_active(PaneId::Header) {
            return self.activate(app);
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