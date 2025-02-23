use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};

use crate::tui::{
    app::{App, AppCommand}, 
    widgets::button::{Button, BLUE},
    navigation::PaneId,
};

use super::pane::{Pane, PaneExt};

pub struct HeaderPane {
    run_query_button: Button<'static>,
}

impl HeaderPane {
    pub fn new() -> Self {
        Self {
            run_query_button: Button::new("Run Query"),
        }
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

    fn is_mouse_event_on_button(&self, mouse_event: MouseEvent) -> bool {
        let terminal_width = crossterm::terminal::size().unwrap_or((80, 24)).0;
    
        let button_width = 15;
        let button_x_start = terminal_width.saturating_sub(button_width + 1);
        let button_x_end = terminal_width.saturating_sub(2);
    
        mouse_event.column >= button_x_start && mouse_event.column <= button_x_end
    }
}

impl Pane for HeaderPane {
    fn pane_id(&self) -> PaneId {
        PaneId::Header
    }

    fn title(&self) -> &'static str {
        "Connection"
    }

    // TODO: Fix the layout
    fn render_content(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),    // App title bar
                Constraint::Min(1),       // Connection area
            ])
            .split(area);
        
        let app_info_line = Line::from(vec![
            " sqli ".white().bold(),
            "v0.1.0".dark_gray().into(),
        ]);
        
        frame.render_widget(
            Paragraph::new(app_info_line).style(Style::default()),
            chunks[0]
        );

        let is_editing = app.is_pane_in_edit_mode(self.pane_id());
        let prefix = if is_editing { "◀ " } else { "" };
        let suffix = if is_editing { " ▶" } else { "" };
        
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
            chunks[1]
        );

        self.render_connection_button(frame, chunks[1]);
    }

    fn get_custom_instructions(&self, _app: &App, is_editing: bool) -> Line<'static> {
        if is_editing {
            Line::from(vec![
                " Esc ".blue().bold(),
                "Return ".white().into(),
                " ←/→ ".blue().bold(),
                "Change Connection ".white().into(),
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

    fn handle_edit_mode_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        match key.code {
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
    }

    fn handle_active_mode_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate(app)
            },
            KeyCode::Down => {
                app.navigation.activate_pane(PaneId::Collections)?;
                Ok(false)
            },
            _ => Ok(false)
        }
    }

    fn handle_custom_mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.is_mouse_event_on_button(mouse_event) {
                    app.query_state.pending_command = AppCommand::ExecuteQuery;
                    return Ok(true);
                }
                Ok(false)
            },
            _ => {
                self.run_query_button.handle_mouse_event(mouse_event);
                Ok(false)
            }
        }
    }
}