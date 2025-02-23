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

use crate::{collection::{build_collection_tree, load_collections}, file::load_sql_content, tui::widgets::file_tree::FileTree};
use crate::tui::app::{App, Mode};
use crate::tui::navigation::{Navigable, PaneId, FocusType};
use super::traits::Instructions;

pub struct CollectionsPane {
    last_area: Option<Rect>,
}

impl CollectionsPane {
    pub fn new() -> Self {
        Self {
            last_area: None,
        }
    }

    pub fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        self.last_area = Some(area);

        let focus_type = if let Some(info) = app.navigation.get_pane_info(PaneId::Collections) {
            info.focus_type
        } else {
            FocusType::Inactive
        };

        let focus_style = match focus_type {
            FocusType::Editing => Style::default().fg(Color::LightBlue).bold(),
            FocusType::Active => Style::default().fg(Color::LightBlue),
            FocusType::Inactive => Style::default().fg(Color::White),
        };

        let tree = FileTree::new(&app.ui_state.collection_items)
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

        frame.render_stateful_widget(tree, area, &mut app.ui_state.collection_state);
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
        let selected = app.ui_state.collection_state.selected();
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
                app.ui_state.workspace.clear();
                app.ui_state.workspace.insert_str(&content);
            },
            Err(err) => {
                app.ui_state.message = format!("Error loading SQL file: {}", err);
            },
        }
        return Ok(());
    }
}

impl Instructions for CollectionsPane {
    fn get_instructions(&self, app: &App) -> Line {
        if app.mode != Mode::Normal {
            return Line::from("");
        }
        
        if !app.is_collections_active() {
            return Line::from("");
        }
        
        if app.is_pane_in_edit_mode(PaneId::Collections) {
            Line::from(vec![
                " Esc ".blue().bold(),
                "Return ".white().into(),
                " ↑/↓ ".blue().bold(),
                "Navigate ".white().into(),
                " Space ".blue().bold(),
                "Confirm ".white().into(),
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

impl Navigable for CollectionsPane {
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        if app.mode != Mode::Normal || !app.navigation.is_active(PaneId::Collections) {
            return Ok(false);
        }
        
        let info = app.navigation.get_pane_info(PaneId::Collections).unwrap();
        match info.focus_type {
            FocusType::Active => {
                match key_event.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.activate(app)
                    },
                    KeyCode::Up => {
                        app.navigation.activate_pane(PaneId::Header)?;
                        Ok(false)
                    },
                    KeyCode::Right => {
                        app.navigation.activate_pane(PaneId::Workspace)?;
                        Ok(false)
                    }
                    _ => Ok(false)
                }
            },
            FocusType::Editing => {
                match key_event.code {
                    KeyCode::Esc => {
                        self.deactivate(app)
                    },
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        app.ui_state.collection_state.toggle_selected();
                        self.handle_selection(app)?;
                        Ok(false)
                    },
                    KeyCode::Left => {
                        app.ui_state.collection_state.key_left();
                        Ok(false)
                    },
                    KeyCode::Right => {
                        app.ui_state.collection_state.key_right();
                        Ok(false)
                    },
                    KeyCode::Down => {
                        app.ui_state.collection_state.key_down();
                        Ok(false)
                    },
                    KeyCode::Up => {
                        app.ui_state.collection_state.key_up();
                        Ok(false)
                    },
                    _ => Ok(false)
                }
            },
            _ => Ok(false)
        }
    }
    
    fn handle_mouse_event(&self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        app.navigation.activate_pane(PaneId::Collections)?;

        if let Some(area) = self.last_area {
            let tree = FileTree::new(&app.ui_state.collection_items).expect("all item identifiers are unique");
            if tree.handle_mouse_event(&mut app.ui_state.collection_state, mouse_event, area)? {
                app.navigation.start_editing(PaneId::Collections)?;
                self.handle_selection(app)?;
                return Ok(false);
            }
        }
        
        app.navigation.start_editing(PaneId::Collections)?;
        Ok(false)
    }
    
    fn activate(&self, app: &mut App) -> Result<bool> {
        app.navigation.start_editing(PaneId::Collections)?;
        Ok(false)
    }
    
    fn deactivate(&self, app: &mut App) -> Result<bool> {
        app.navigation.stop_editing(PaneId::Collections)?;
        Ok(false)
    }
}