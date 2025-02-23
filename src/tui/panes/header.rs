use anyhow::{Ok, Result};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
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

pub struct HeaderPane {
    run_query_button: Button<'static>,
}

impl HeaderPane {
    pub fn new() -> Self {
        Self {
            run_query_button: Button::new("Run Query"),
        }
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

        let is_editing = focus_type == FocusType::Editing;
        let prefix = if is_editing { "◀ " } else { "" };
        let suffix = if is_editing { " ▶" } else { "" };
        
        let connection_block = Block::default()
            .title("Connection")
            .title_style(focus_style)
            .borders(Borders::ALL)
            .border_style(focus_style);
        
        frame.render_widget(&connection_block, conn_area);

        let connection_name = match app.get_current_connection() {
            Some(name) => format!("{prefix}{name}{suffix}"),
            None => "No connection selected".to_string(),
        };

        let connection_style = if is_editing {
            Style::default().fg(Color::LightBlue).bold()
        } else {
            Style::default()
        };

        frame.render_widget(
            Paragraph::new(connection_name)
                .style(connection_style)
                .alignment(Alignment::Left),
            connection_block.inner(conn_area)
        );

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
    
        self.run_query_button.set_area(horizontal[1]);
        frame.render_widget(
            self.run_query_button.clone().theme(BLUE),
            horizontal[1]
        );
    }
}

impl Instructions for HeaderPane {
    fn get_instructions(&self, app: &App) -> Line<'static> {
        if app.mode != Mode::Normal {
            return Line::from("");
        }
        
        if !app.is_header_active() {
            return Line::from("");
        }
        
        if app.is_pane_in_edit_mode(PaneId::Header) {
            Line::from(vec![
                " Esc ".blue().bold(),
                "Return ".white().into(),
                " ←/→ ".blue().bold(),
                " Change Connection ".white().into(),
                " ^C ".blue().bold(),
                "Quit ".white().into(),
            ])
        } else {
            Line::from(vec![
                " Tab ".blue().bold(),
                "Switch Panel ".white().into(),
                " Space ".blue().bold(),
                "Select ".white().into(),
                " ^C ".blue().bold(),
                "Quit ".white().into(),
            ])
        }
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
                    KeyCode::Left => {
                        app.previous_connection();
                        Ok(false)
                    },
                    KeyCode::Right => {
                        app.next_connection();
                        Ok(false)
                    },
                    _ => Ok(false)
                }
            },
            _ => Ok(false)
        }
    }
    
    fn handle_mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if is_mouse_event_on_button(mouse_event) {
                    app.query_state.pending_command = AppCommand::ExecuteQuery;
                    return Ok(false);
                }

                if app.navigation.is_active(PaneId::Header) {
                    return self.activate(app);
                }
                app.navigation.activate_pane(PaneId::Header)?;
                return Ok(false);
            },
            _ => {
                self.run_query_button.handle_mouse_event(mouse_event);
                return Ok(false);
             }
        }
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

fn is_mouse_event_on_button(mouse_event: MouseEvent) -> bool {
    let terminal_width = crossterm::terminal::size().unwrap_or((80, 24)).0;

    let button_width = 15;
    let button_x_start = terminal_width.saturating_sub(button_width + 1);
    let button_x_end = terminal_width.saturating_sub(2);

    mouse_event.column >= button_x_start && mouse_event.column <= button_x_end
}