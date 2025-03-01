use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    text::Line,
    widgets::{Scrollbar, ScrollbarOrientation},
    Frame,
};

use crate::{
    collection::SelectedFile,
    config::CONFIG_FILE_NAME,
    file::{self, FileSystem},
    tui::{
        widgets::file_tree::FileTree,
        navigation::PaneId,
        app::App,
    },
};

use super::pane::{Pane, PaneExt};

pub struct CollectionsPane {
    last_area: Option<Rect>,
}

impl CollectionsPane {
    pub fn new() -> Self {
        Self {
            last_area: None,
        }
    }

    pub fn handle_selection(&self, app: &mut App) -> anyhow::Result<()> {
        let selected = app.ui_state.collection_state.selected();
        if selected.is_empty() {
            return Ok(());
        }

        let fs = FileSystem::new()?;
        let file = match file::parse_selected_file(&selected) {
            Some(file) => file,
            None => return Ok(())
        };

        let result = match file {
            SelectedFile::Config(scope) => {
                let config_path = fs.get_scoped_path(scope, CONFIG_FILE_NAME)?;
                fs.read_file(config_path)
            },
            SelectedFile::Sql { collection, filename, scope } => {
                fs.load_sql(&collection, &filename, scope)
            },
            // Do nothing for folders
            _ => return Ok(())
        };

        match result {
            Ok(content) => {
                app.ui_state.workspace.clear();
                app.ui_state.workspace.insert_str(&content);
            },
            Err(err) => {
                app.ui_state.message = format!("Error loading file: {}", err);
            }
        }

        Ok(())
    }
}

impl Pane for CollectionsPane {
    fn pane_id(&self) -> PaneId {
        PaneId::Collections
    }

    fn title(&self) -> &'static str {
        "Collections"
    }

    fn render_content(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        self.last_area = Some(area);

        let tree = FileTree::new(&app.ui_state.collection_items)
            .expect("all item identifiers are unique")
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

    fn get_custom_instructions(&self, _app: &App, is_editing: bool) -> Line<'static> {
        if is_editing {
            Line::from(vec![
                " Esc ".blue().bold(),
                "Return ".white().into(),
                " ↑/↓ ".blue().bold(),
                "Navigate ".white().into(),
                " Space ".blue().bold(),
                "Confirm ".white().into(),
                " ^N ".blue().bold(),
                "New ".white().into(),
                " ^E ".blue().bold(),
                "Edit ".white().into(),
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
    }

    fn handle_active_mode_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        match key.code {
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
            },
            _ => Ok(false)
        }
    }

    fn handle_custom_mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
            if let Some(area) = self.last_area {
                let tree = FileTree::new(&app.ui_state.collection_items).expect("all item identifiers are unique");
                
                if tree.handle_mouse_event(&mut app.ui_state.collection_state, mouse_event, area)? {
                    app.navigation.start_editing(PaneId::Collections)?;                    
                    self.handle_selection(app)?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}