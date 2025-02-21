use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Scrollbar, ScrollbarOrientation},
    Frame,
};
use tui_tree_widget::Tree;

use crate::collection::{build_collection_tree, load_collections, load_sql_content};
use super::app::{App, Focus};

pub struct CollectionsPane;

impl CollectionsPane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect) {
        let focus_style = if app.focus == Focus::Collections {
            Style::default().fg(Color::LightBlue)// .bold()
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
                    // .bold()
            )
            .highlight_symbol("➤ ")
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None)
            ));

        // Render the tree as a stateful widget
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