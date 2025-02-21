use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

use super::app::{App, Focus};

pub struct WorkspacePane;

impl WorkspacePane {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame, area: Rect, search_height: u16) {
        let workspace_focus = if app.focus == Focus::WorkspaceEdit {
            Style::default().fg(Color::LightBlue) //.bold()
        } else if app.focus == Focus::Workspace {
            Style::default().fg(Color::LightBlue)// .bold()
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