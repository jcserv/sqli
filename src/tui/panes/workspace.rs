use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    text::Line,
    Frame,
};

use crate::tui::{
    app::{App, AppCommand},
    navigation::PaneId,
};

use super::pane::{Pane, PaneExt};

pub struct WorkspacePane;

impl Default for WorkspacePane {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspacePane {
    pub fn new() -> Self {
        Self
    }

    pub fn update_dimensions(&self, app: &mut App, height: u16) {
        app.ui_state.workspace.update_dimensions(height);
    }
}

impl Pane for WorkspacePane {
    fn pane_id(&self) -> PaneId {
        PaneId::Workspace
    }

    fn title(&self) -> &'static str {
        "Workspace"
    }

    fn render_content(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let search_height = if app.ui_state.search.open { 3 } else { 0 };
        
        let workspace_widget = app.ui_state.workspace.clone();
        
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
        
        frame.render_widget(&workspace_widget, workspace_area);
    }

    fn get_custom_instructions(&self, _app: &App, is_editing: bool) -> Line<'static> {
        if is_editing {
            Line::from(vec![
                " Esc ".blue().bold(),
                "Return ".white(),
                " ^S ".blue().bold(),
                "Save ".white(),
                " ^Space ".blue().bold(),
                "Run ".white(),
                " ^C ".blue().bold(),
                "Quit ".white(),
            ])
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
        match key.code {
            KeyCode::Esc => {
                self.deactivate(app)
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.save_query();
                Ok(false)
            }
            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.query_state.pending_command = AppCommand::ExecuteQuery;
                self.deactivate(app)?;
                Ok(false)
            }
            _ => {
                let input = tui_textarea::Input::from(key);
                app.ui_state.workspace.input(input);
                Ok(false)
            }
        }
    }

    fn handle_active_mode_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        match key.code {
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
}