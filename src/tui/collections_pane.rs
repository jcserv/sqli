use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    prelude::*,
    Frame,
    layout::Rect, 
    style::{Color, Style}, 
    text::Line, 
    widgets::{Block, Borders, Scrollbar, ScrollbarOrientation},
};
use tui_tree_widget::Tree;

use crate::collection::{build_collection_tree, load_collections, load_sql_content};
use super::{app::{App, Focus, Mode}, traits::{Instructions, PaneEventHandler}};

pub struct CollectionsPane;

impl CollectionsPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect) {
        let focus_style = if app.focus == Focus::CollectionsEdit {
            Style::default().fg(Color::LightBlue).bold()
        } else if app.focus == Focus::Collections {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default().fg(Color::White)
        };

        let tree = Tree::new(&app.collection_items)
            .expect("all item identifiers are unique")
            .block(
                Block::default()
                    .title("Collections")
                    .title_style(focus_style)
                    .borders(Borders::ALL)
                    .border_style(focus_style)
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightBlue)
                    .bold()
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None)
            ));

        frame.render_stateful_widget(tree, area, &mut app.collection_state);
    }

    pub fn load_collections() -> Vec<tui_tree_widget::TreeItem<'static, String>> {
        match load_collections() {
            Ok(cols) => build_collection_tree(&cols),
            Err(err) => {
                eprintln!("Error loading collections: {}", err);
                Vec::new()
            }
        }
    }

    pub fn handle_selection(&self, app: &mut App) -> anyhow::Result<()> {
        let selected = app.collection_state.selected();
        if selected.is_empty() {
            return Ok(());
        }

        let path = selected.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        if path.len() < 2 {
            return Ok(());
        }
            
        let collection_name = path[0];
        let file_name = path[path.len() - 1];

        if !file_name.ends_with(".sql") {
            return Ok(());
        }
            
        match load_sql_content(collection_name, file_name) {
            Ok(content) => {
                app.workspace.clear();
                app.workspace.insert_str(&content);
                
                // Switch to workspace edit mode
                app.focus = Focus::WorkspaceEdit;
                app.current_tab = super::app::Tab::Workspace;
                
                app.message = format!("Loaded {}/{}", collection_name, file_name);
            },
            Err(err) => {
                app.message = format!("Error loading SQL file: {}", err);
            },
        }
        return Ok(());
    }
}

impl Instructions for CollectionsPane {
    fn get_instructions(&self, app: &App) -> Line {
        match app.mode {
            Mode::Normal => {
                match app.focus {
                    Focus::Collections => {
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
                    },
                    Focus::CollectionsEdit => {
                        Line::from(vec![
                            " Esc ".blue().bold(),
                            "Stop Editing ".white().into(),
                            " ↑/↓ ".blue().bold(),
                            "Navigate ".white().into(),
                            " Enter ".blue().bold(),
                            "Collapse/Expand ".white().into(),
                            " ^P ".blue().bold(),
                            "Command ".white().into(),
                            " ^C ".blue().bold(),
                            "Quit ".white().into(),
                        ])
                    },
                    _ => Line::from(""),
                }
            },
            _ => Line::from(""),
        }
    }
}

impl PaneEventHandler for CollectionsPane {
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        if app.mode != Mode::Normal {
            return Ok(false);
        }
        
        match app.focus {
            Focus::Collections => {
                match key_event.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        app.focus = Focus::CollectionsEdit;
                        Ok(false)
                    },
                    _ => Ok(false)
                }
            },
            Focus::CollectionsEdit => {
                match key_event.code {
                    KeyCode::Esc => {
                        app.focus = Focus::Collections;
                        Ok(false)
                    },
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        app.collection_state.toggle_selected();
                        self.handle_selection(app)?;
                        Ok(false)
                    },
                    KeyCode::Left => {
                        app.collection_state.key_left();
                        Ok(false)
                    },
                    KeyCode::Right => {
                        app.collection_state.key_right();
                        Ok(false)
                    },
                    KeyCode::Down => {
                        app.collection_state.key_down();
                        Ok(false)
                    },
                    KeyCode::Up => {
                        app.collection_state.key_up();
                        Ok(false)
                    },
                    _ => Ok(false)
                }
            },
            _ => Ok(false)
        }
    }
    
    fn handle_mouse_event(&self, app: &mut App, _mouse_event: MouseEvent) -> Result<bool> {
        if app.current_tab != super::app::Tab::Collections {
            app.select_tab(super::app::Tab::Collections);
        }
        
        
        if app.focus == Focus::Collections {
            app.focus = Focus::CollectionsEdit;
        } else {
            app.collection_state.toggle_selected();
            self.handle_selection(app)?;
        }
        
        Ok(false)
    }
}