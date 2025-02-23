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
                        // " ^F ".blue().bold(),
                        // "Find ".white().into(),
                        // " ^R ".blue().bold(),
                        // "Replace ".white().into(),
                        // " ^P ".blue().bold(),
                        // "Command ".white().into(),
                        " ^C ".blue().bold(),
                        "Quit ".white().into(),
                    ])
                } else {
                    Line::from(vec![
                        " Tab ".blue().bold(),
                        "Switch Panel ".white().into(),
                        " Space ".blue().bold(),
                        "Select ".white().into(),
                        // " ^F ".blue().bold(),
                        // "Find ".white().into(),
                        // " ^P ".blue().bold(),
                        // "Command ".white().into(),
                        " ^C ".blue().bold(),
                        "Quit ".white().into(),
                    ])
                }
            },
            Mode::Search => {
                if app.ui_state.search.replace_mode {
                    Line::from(vec![
                        " ESC ".blue().bold(),
                        "Cancel ".white().into(),
                        " Enter ".blue().bold(),
                        "Replace ".white().into(),
                        // " ^N ".blue().bold(),
                        // "Next ".white().into(),
                        // " ^P ".blue().bold(),
                        // "Previous ".white().into(),
                        " ^C ".blue().bold(),
                        "Quit ".white().into(),
                    ])
                } else {
                    Line::from(vec![
                        " ESC ".blue().bold(),
                        "Cancel ".white().into(),
                        " Enter ".blue().bold(),
                        "Find ".white().into(),
                        // " ^N ".blue().bold(),
                        // "Next ".white().into(),
                        // " ^P ".blue().bold(),
                        // "Previous ".white().into(),
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
                        KeyCode::Char('f') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.mode = Mode::Search;
                            app.ui_state.search.open = true;
                            app.ui_state.search.replace_mode = false;
                            app.ui_state.search.textarea.move_cursor(tui_textarea::CursorMove::End);
                            app.ui_state.search.textarea.delete_line_by_head();
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
                        KeyCode::Char('f') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.mode = Mode::Search;
                            app.ui_state.search.open = true;
                            app.ui_state.search.replace_mode = false;
                            app.ui_state.search.textarea.move_cursor(tui_textarea::CursorMove::End);
                            app.ui_state.search.textarea.delete_line_by_head();
                            Ok(false)
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
                        // KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        //     app.mode = Mode::Search;
                        //     app.ui_state.search.open = true;
                        //     app.ui_state.search.replace_mode = true;
                        //     app.ui_state.search.textarea.move_cursor(tui_textarea::CursorMove::End);
                        //     app.ui_state.search.textarea.delete_line_by_head();
                        //     Ok(false)
                        // }
                        _ => Ok(false)
                    }
                }
            },
            Mode::Search => {
                // Handle search mode inputs
                let input = tui_textarea::Input::from(key_event);
                match input {
                    tui_textarea::Input { key: tui_textarea::Key::Esc, .. } => {
                        app.ui_state.search.open = false;
                        app.mode = Mode::Normal;
                        app.ui_state.workspace.set_search_pattern("")?;
                        Ok(false)
                    }
                    tui_textarea::Input { key: tui_textarea::Key::Enter, .. } => {
                        if app.ui_state.search.replace_mode {
                            let pattern = app.ui_state.search.textarea.lines()[0].as_str();
                            let replacement = app.ui_state.search.textarea.lines().get(1)
                                                 .map(|s| s.as_str()).unwrap_or("");
                            app.ui_state.workspace.set_search_pattern(pattern)?;
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                let count = app.ui_state.workspace.replace_all(replacement);
                                app.ui_state.message = format!("Replaced {} occurrences", count);
                            } else {
                                if app.ui_state.workspace.replace_next(replacement) {
                                    app.ui_state.message = "Replaced occurrence".to_string();
                                } else {
                                    app.ui_state.message = "No more matches".to_string();
                                }
                            }
                        } else {
                            let pattern = app.ui_state.search.textarea.lines()[0].as_str();
                            app.ui_state.workspace.set_search_pattern(pattern)?;
                            if !app.ui_state.workspace.search_forward(true) {
                                app.ui_state.message = "Pattern not found".to_string();
                            }
                        }
                        app.ui_state.search.open = false;
                        app.mode = Mode::Normal;
                        Ok(false)
                    }
                    tui_textarea::Input { 
                        key: tui_textarea::Key::Char('n'),
                        ctrl: true,
                        ..
                    } => {
                        if !app.ui_state.workspace.search_forward(false) {
                            app.ui_state.message = "Pattern not found".to_string();
                        }
                        Ok(false)
                    }
                    tui_textarea::Input {
                        key: tui_textarea::Key::Char('p'),
                        ctrl: true,
                        ..
                    } => {
                        if !app.ui_state.workspace.search_back(false) {
                            app.ui_state.message = "Pattern not found".to_string();
                        }
                        Ok(false)
                    }
                    _ => {
                        app.ui_state.search.textarea.input(input);
                        if let Some(pattern) = app.ui_state.search.textarea.lines().first() {
                            app.ui_state.workspace.set_search_pattern(pattern)?;
                        }
                        Ok(false)
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