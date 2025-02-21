use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent, KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
    text::Line,
    Frame,
};

use crate::tui::{app::{App, Focus, Mode, Tab}, traits::{Instructions, PaneEventHandler}};

pub struct WorkspacePane;

impl WorkspacePane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect, search_height: u16) {
        let workspace_focus = if app.focus == Focus::WorkspaceEdit {
            Style::default().fg(Color::LightBlue).bold()
        } else if app.focus == Focus::Workspace {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title("Workspace")
            .title_style(workspace_focus)
            .borders(Borders::ALL)
            .border_style(workspace_focus);

        let mut workspace_widget = app.workspace.clone();
        workspace_widget.set_block(block);
        
        if !app.search.open {
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
        
        frame.render_widget(&app.search.textarea, search_area);
        frame.render_widget(&workspace_widget, workspace_area);
    }

    pub fn update_dimensions(&self, app: &mut App, height: u16) {
        app.workspace.update_dimensions(height);
    }
}

impl Instructions for WorkspacePane {
    fn get_instructions(&self, app: &App) -> Line<'static> {
        match app.mode {
            Mode::Normal => {
                match app.focus {
                    Focus::Workspace => {
                        Line::from(vec![
                            " Tab ".blue().bold(),
                            "Switch Panel ".white().into(),
                            " Space ".blue().bold(),
                            "Edit ".white().into(),
                            " ^F ".blue().bold(),
                            "Find ".white().into(),
                            " ^R ".blue().bold(),
                            "Replace ".white().into(),
                            " ^P ".blue().bold(),
                            "Command ".white().into(),
                            " ^C ".blue().bold(),
                            "Quit ".white().into(),
                        ])
                    },
                    Focus::WorkspaceEdit => {
                        Line::from(vec![
                            " Esc ".blue().bold(),
                            "Stop Editing ".white().into(),
                            " ^S ".blue().bold(),
                            "Save ".white().into(),
                            " ^F ".blue().bold(),
                            "Find ".white().into(),
                            " ^R ".blue().bold(),
                            "Replace ".white().into(),
                            " ^P ".blue().bold(),
                            "Command ".white().into(),
                            " ^C ".blue().bold(),
                            "Quit ".white().into(),
                        ])
                    },
                    _ => Line::from(""),
                }
            },
            Mode::Search => {
                if app.search.replace_mode {
                    Line::from(vec![
                        " ESC ".blue().bold(),
                        "Cancel ".white().into(),
                        " Enter ".blue().bold(),
                        "Replace ".white().into(),
                        " ^N ".blue().bold(),
                        "Next ".white().into(),
                        " ^P ".blue().bold(),
                        "Previous ".white().into(),
                        " ^C ".blue().bold(),
                        "Quit ".white().into(),
                    ])
                } else {
                    Line::from(vec![
                        " ESC ".blue().bold(),
                        "Cancel ".white().into(),
                        " Enter ".blue().bold(),
                        "Find ".white().into(),
                        " ^N ".blue().bold(),
                        "Next ".white().into(),
                        " ^P ".blue().bold(),
                        "Previous ".white().into(),
                        " ^C ".blue().bold(),
                        "Quit ".white().into(),
                    ])
                }
            },
            _ => Line::from(""),
        }
    }
}

impl PaneEventHandler for WorkspacePane {
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        match app.mode {
            Mode::Normal => {
                match app.focus {
                    Focus::Workspace => {
                        match key_event.code {
                            KeyCode::Char(' ') | KeyCode::Enter => {
                                app.focus = Focus::WorkspaceEdit;
                                Ok(false)
                            }
                            KeyCode::Char('f') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.mode = Mode::Search;
                                app.search.open = true;
                                app.search.replace_mode = false;
                                app.search.textarea.move_cursor(tui_textarea::CursorMove::End);
                                app.search.textarea.delete_line_by_head();
                                Ok(false)
                            }
                            KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.mode = Mode::Search;
                                app.search.open = true;
                                app.search.replace_mode = true;
                                app.search.textarea.move_cursor(tui_textarea::CursorMove::End);
                                app.search.textarea.delete_line_by_head();
                                Ok(false)
                            }
                            _ => Ok(false)
                        }
                    },
                    Focus::WorkspaceEdit => {
                        match key_event.code {
                            KeyCode::Esc => {
                                app.focus = Focus::Workspace;
                                Ok(false)
                            }
                            KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.save_query();
                                Ok(false)
                            }
                            _ => {
                                // Forward all other keys to the text area
                                let input = tui_textarea::Input::from(key_event);
                                app.workspace.input(input);
                                Ok(false)
                            }
                        }
                    },
                    _ => Ok(false),
                }
            },
            Mode::Search => {
                // Handle search mode, which is specific to the workspace pane
                let input = tui_textarea::Input::from(key_event);
                match input {
                    tui_textarea::Input { key: tui_textarea::Key::Esc, .. } => {
                        app.search.open = false;
                        app.mode = Mode::Normal;
                        app.workspace.set_search_pattern("")?;
                        Ok(false)
                    }
                    tui_textarea::Input { key: tui_textarea::Key::Enter, .. } => {
                        if app.search.replace_mode {
                            let pattern = app.search.textarea.lines()[0].as_str();
                            let replacement = app.search.textarea.lines().get(1).map(|s| s.as_str()).unwrap_or("");
                            app.workspace.set_search_pattern(pattern)?;
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                let count = app.workspace.replace_all(replacement);
                                app.message = format!("Replaced {} occurrences", count);
                            } else {
                                if app.workspace.replace_next(replacement) {
                                    app.message = "Replaced occurrence".to_string();
                                } else {
                                    app.message = "No more matches".to_string();
                                }
                            }
                        } else {
                            let pattern = app.search.textarea.lines()[0].as_str();
                            app.workspace.set_search_pattern(pattern)?;
                            if !app.workspace.search_forward(true) {
                                app.message = "Pattern not found".to_string();
                            }
                        }
                        app.search.open = false;
                        app.mode = Mode::Normal;
                        Ok(false)
                    }
                    tui_textarea::Input { 
                        key: tui_textarea::Key::Char('n'),
                        ctrl: true,
                        ..
                    } => {
                        if !app.workspace.search_forward(false) {
                            app.message = "Pattern not found".to_string();
                        }
                        Ok(false)
                    }
                    tui_textarea::Input {
                        key: tui_textarea::Key::Char('p'),
                        ctrl: true,
                        ..
                    } => {
                        if !app.workspace.search_back(false) {
                            app.message = "Pattern not found".to_string();
                        }
                        Ok(false)
                    }
                    _ => {
                        app.search.textarea.input(input);
                        if let Some(pattern) = app.search.textarea.lines().first() {
                            app.workspace.set_search_pattern(pattern)?;
                        }
                        Ok(false)
                    }
                }
            },
            _ => Ok(false),
        }
    }
    
    fn handle_mouse_event(&self, app: &mut App, _mouse_event: MouseEvent) -> Result<bool> {
        app.select_tab(Tab::Workspace);        
        if app.focus == Focus::Workspace {
            app.focus = Focus::WorkspaceEdit;
        }        
        Ok(false)
    }
}