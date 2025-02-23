use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent, MouseButton, MouseEventKind};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    text::Line, widgets::{Block, Borders},
};

use crate::tui::{
    app::{App, Mode},
    navigation::{FocusType, PaneId},
};

/// Core trait that all panes must implement
pub trait Pane {
    /// Get the unique identifier for this pane
    fn pane_id(&self) -> PaneId;
    
    /// Get the title for this pane
    fn title(&self) -> &'static str;
    
    /// Render the pane's content
    fn render_content(&mut self, app: &mut App, frame: &mut Frame, area: Rect);
    
    /// Get instructions specific to this pane
    fn get_custom_instructions(&self, app: &App, is_editing: bool) -> Line<'static>;
    
    /// Handle custom activation logic
    fn handle_activate(&mut self, _app: &mut App) -> Result<bool> {
        Ok(false)
    }
    
    /// Handle custom deactivation logic  
    fn handle_deactivate(&mut self, _app: &mut App) -> Result<bool> {
        Ok(false)
    }
    
    /// Handle custom key events when in edit mode
    fn handle_edit_mode_key(&mut self, _app: &mut App, _key: KeyEvent) -> Result<bool> {
        Ok(false)
    }
    
    /// Handle custom key events when in active mode
    fn handle_active_mode_key(&mut self, _app: &mut App, _key: KeyEvent) -> Result<bool> {
        Ok(false)
    }

    /// Handle custom mouse events
    fn handle_custom_mouse_event(&mut self, _app: &mut App, _mouse_event: MouseEvent) -> Result<bool> {
        Ok(false)
    }
}

/// Extension trait that provides common pane functionality
pub trait PaneExt: Pane {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let focus_type = if let Some(info) = app.navigation.get_pane_info(self.pane_id()) {
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
            .title(self.title())
            .title_style(focus_style)
            .borders(Borders::ALL)
            .border_style(focus_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        
        self.render_content(app, frame, inner_area);
    }

    fn get_instructions(&self, app: &App) -> Line<'static> {
        if app.mode != Mode::Normal {
            return Line::from("");
        }
        
        if !app.navigation.is_active(self.pane_id()) {
            return Line::from("");
        }
        
        let is_editing = app.is_pane_in_edit_mode(self.pane_id());
        self.get_custom_instructions(app, is_editing)
    }

    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        if app.mode != Mode::Normal || !app.navigation.is_active(self.pane_id()) {
            return Ok(false);
        }

        let info = app.navigation.get_pane_info(self.pane_id()).unwrap();
        match info.focus_type {
            FocusType::Active => {
                self.handle_active_mode_key(app, key_event)
            },
            FocusType::Editing => {
                self.handle_edit_mode_key(app, key_event)
            },
            _ => Ok(false)
        }
    }

    fn handle_mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if app.navigation.is_active(self.pane_id()) {
                    return self.handle_activate(app);
                }
                app.navigation.activate_pane(self.pane_id())?;
                self.handle_activate(app)
            },
            _ => Ok(false)
        }
    }

    fn activate(&mut self, app: &mut App) -> Result<bool> {
        app.navigation.start_editing(self.pane_id())?;
        self.handle_activate(app)
    }

    fn deactivate(&mut self, app: &mut App) -> Result<bool> {
        app.navigation.stop_editing(self.pane_id())?;
        self.handle_deactivate(app)
    }
}

// Automatically implement PaneExt for any type that implements Pane
impl<T: Pane> PaneExt for T {}