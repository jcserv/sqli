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
    app::{App, AppCommand}, navigation::PaneId, widgets::button::{Button, State, BLUE}
};

use super::pane::{Pane, PaneExt};

pub struct HeaderPane {
    run_query_button: Button<'static>,
}

impl Default for HeaderPane {
    fn default() -> Self {
        Self::new()
    }
}

impl HeaderPane {
    pub fn new() -> Self {
        Self {
            run_query_button: Button::new("Run Query"),
        }
    }

    fn get_focused_element(&self, app: &App) -> usize {
        if let Some(info) = app.navigation.get_pane_info(self.pane_id()) {
            info.current_element
        } else {
            0
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

    fn render_content(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),       // Connection area
            ])
            .split(area);

        let focused_element = self.get_focused_element(app);
        let is_editing = app.is_pane_in_edit_mode(self.pane_id());

        let connection_style = if is_editing && focused_element == 0 {
            Style::default().fg(Color::LightBlue).bold()
        } else if focused_element == 0 {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default()
        };

        let prefix = if is_editing { "◀ " } else { "" };
        let suffix = if is_editing { " ▶" } else { "" };

        let connection_name = match app.get_current_connection() {
            Some(name) => format!("{prefix}{name}{suffix}"),
            None => "No connection selected".to_string(),
        };

        if is_editing && focused_element == 1 {
            self.run_query_button.set_state(State::Selected);
        } else if focused_element == 1 {
            self.run_query_button.set_state(State::Hover);
        } else {
            self.run_query_button.set_state(State::Normal);
        }

        frame.render_widget(
            Paragraph::new(connection_name)
                .style(connection_style)
                .alignment(Alignment::Left),
            chunks[0]
        );

        self.render_connection_button(frame, chunks[0]);
    }

    fn get_custom_instructions(&self, app: &App, is_editing: bool) -> Line<'static> {
        let focused_element = self.get_focused_element(app);

        if is_editing {
            if focused_element == 0 {
                Line::from(vec![
                    " Esc ".blue().bold(),
                    "Return ".white(),
                    " Tab ".blue().bold(),
                    "Next Element ".white(),
                    " ←/→ ".blue().bold(),
                    "Change Connection ".white(),
                    " ^C ".blue().bold(),
                    "Quit ".white(),
                ])
            } else {
                Line::from(vec![
                    " Esc ".blue().bold(),
                    "Return ".white(),
                    " Tab ".blue().bold(),
                    "Next Element ".white(),
                    " Space ".blue().bold(),
                    "Run Query ".white(),
                    " ^C ".blue().bold(),
                    "Quit ".white(),
                ])
            }
        } else {
            Line::from(vec![
                " Tab ".blue().bold(),
                "Switch Panel ".white(),
                " Space ".blue().bold(),
                "Select ".white(),
                " ^C ".blue().bold(),
                "Quit ".white(),
            ])
        }
    }

    fn handle_edit_mode_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        let focused_element = self.get_focused_element(app);

        match key.code {
            KeyCode::Esc => {
                self.deactivate(app)
            },
            KeyCode::BackTab => {
                if let Some(info) = app.navigation.get_pane_info_mut(self.pane_id()) {
                    info.prev_element();
                }
                Ok(false)
            },
            KeyCode::Tab => {
                if let Some(info) = app.navigation.get_pane_info_mut(self.pane_id()) {
                    info.next_element();
                }
                Ok(false)
            },
            KeyCode::Enter | KeyCode::Char(' ') => {
                if focused_element == 1 {
                    app.query_state.pending_command = AppCommand::ExecuteQuery;
                    Ok(false)
                } else {
                    Ok(false)
                }
            },
            KeyCode::Left => {
                if focused_element == 0 {
                    app.previous_connection();
                }
                Ok(false)
            },
            KeyCode::Right => {
                if focused_element == 0 {
                    app.next_connection();
                }
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
                    return Ok(false);
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