use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
    text::Line,
    Frame,
};

use crate::tui::app::{App, AppCommand, Mode};
use crate::tui::navigation::{Navigable, PaneId, FocusType};
use super::traits::Instructions;

pub struct WorkspacePane;

impl WorkspacePane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect, search_height: u16) {
        let focus_type = if let Some(info) = app.navigation.get_pane_info(PaneId::Workspace) {
            info.focus_type
        } else {
            FocusType::Inactive
        };

        let workspace_focus = match focus_type {
            FocusType::Editing => Style::default().fg(Color::LightBlue).bold(),
            FocusType::Active => Style::default().fg(Color::LightBlue),
            FocusType::Inactive => Style::default().fg(Color::White),
        };

        let block = Block::default()
            .title("Workspace")
            .title_style(workspace_focus)
            .borders(Borders::ALL)
            .border_style(workspace_focus);

        let mut workspace_widget = app.ui_state.workspace.clone();
        workspace_widget.set_block(block);
        
        if !app.ui_state.search.open {
            frame.render_widget(&workspace_widget, area);
            return;
        }
        
        let workspace_area = Rect::new(
            area.x, 
            area.y + search_height, 
            area.width, 
            area.height.saturating_sub(search_height)
        );
        
        let search_area = Rect::new(
            area.x,
            area.y,
            area.width,
            search_height
        );
        
        frame.render_widget(&app.ui_state.search.textarea, search_area);
        frame.render_widget(&workspace_widget, workspace_area);
    }

    pub fn update_dimensions(&self, app: &mut App, height: u16) {
        app.ui_state.workspace.update_dimensions(height);
    }
}

impl Instructions for WorkspacePane {
    fn get_instructions(&self, app: &App) -> Line<'static> {
        match app.mode {
            Mode::Normal => {
                if !app.navigation.is_active(PaneId::Workspace) {
                    return Line::from("");
                }
                
                let is_editing = app.is_pane_in_edit_mode(PaneId::Workspace);
                if is_editing {
                    Line::from(vec![
                        " Esc ".blue().bold(),
                        "Return ".white().into(),
                        " ^S ".blue().bold(),
                        "Save ".white().into(),
                        " ^Space ".blue().bold(),
                        "Run ".white().into(),
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
            },
            _ => Line::from(""),
        }
    }
}

impl Navigable for WorkspacePane {    
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        if !app.navigation.is_active(PaneId::Workspace) {
            return Ok(false);
        }
        match app.mode {
            Mode::Normal => {
                if app.is_pane_in_edit_mode(PaneId::Workspace) {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.deactivate(app)
                        }
                        KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.save_query();
                            Ok(false)
                        }
                        // TODO: I would like this to be a Ctrl+Enter shortcut, but it doesn't work
                        // see: https://github.com/crossterm-rs/crossterm/issues/685
                        KeyCode::Char(' ') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.query_state.pending_command = AppCommand::ExecuteQuery;
                            self.deactivate(app)?;
                            Ok(false)
                        }
                        _ => {
                            let input = tui_textarea::Input::from(key_event);
                            app.ui_state.workspace.input(input);
                            Ok(false)
                        }
                    }
                } else {
                    match key_event.code {
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            self.activate(app)
                        }
                        KeyCode::Up => {
                            app.navigation.activate_pane(PaneId::Header)?;
                            Ok(false)
                        }
                        KeyCode::Left => {
                            app.navigation.activate_pane(PaneId::Collections)?;
                            Ok(false)
                        }
                        KeyCode::Down => {
                            app.navigation.activate_pane(PaneId::Results)?;
                            Ok(false)
                        }
                        _ => Ok(false)
                    }
                }
            },
            _ => Ok(false),
        }
    }
    
    fn handle_mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                app.navigation.activate_pane(PaneId::Workspace)?;
                app.navigation.start_editing(PaneId::Workspace)?;
                Ok(false)
            },
            _ => { Ok(false) }
        }
    }
    
    fn activate(&self, app: &mut App) -> Result<bool> {
        app.navigation.start_editing(PaneId::Workspace)?;
        Ok(false)
    }
    
    fn deactivate(&self, app: &mut App) -> Result<bool> {
        app.navigation.stop_editing(PaneId::Workspace)?;
        Ok(false)
    }
}