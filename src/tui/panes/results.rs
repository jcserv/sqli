use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent, KeyCode};
use ratatui::{
    layout::Rect,
    prelude::*,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders},
    Frame,
};

use crate::tui::app::{App, Mode};
use crate::tui::navigation::{Navigable, PaneId, FocusType};
use super::traits::Instructions;

pub struct ResultsPane;

impl ResultsPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect) {
        // let is_active = app.navigation.is_active(PaneId::Results);
        let focus_type = if let Some(info) = app.navigation.get_pane_info(PaneId::Results) {
            info.focus_type
        } else {
            FocusType::Inactive
        };

        let focus_style = match focus_type {
            FocusType::Editing => Style::default().fg(Color::LightBlue).bold(),
            FocusType::Active => Style::default().fg(Color::LightBlue),
            FocusType::Inactive => Style::default().fg(Color::White),
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
        if app.mode != Mode::Normal || !app.navigation.is_active(PaneId::Results) {
            return Line::from("");
        }
        
        let is_editing = app.is_pane_in_edit_mode(PaneId::Results);
        if is_editing {
            Line::from(vec![
                " Esc ".blue().bold(),
                "Stop Editing ".white().into(),
                " ^P ".blue().bold(),
                "Command ".white().into(),
                " ^C ".blue().bold(),
                "Quit ".white().into(),
            ])
        } else {
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
}

impl Navigable for ResultsPane {
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        if app.mode != Mode::Normal || !app.navigation.is_active(PaneId::Results) {
            return Ok(false);
        }
        
        let is_editing = app.is_pane_in_edit_mode(PaneId::Results);
        
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
        app.navigation.activate_pane(PaneId::Results)?;        
        app.navigation.start_editing(PaneId::Results)?;
        Ok(false)
    }
    
    fn activate(&self, app: &mut App) -> Result<bool> {
        app.navigation.start_editing(PaneId::Results)?;
        Ok(false)
    }
    
    fn deactivate(&self, app: &mut App) -> Result<bool> {
        app.navigation.stop_editing(PaneId::Results)?;
        Ok(false)
    }
}