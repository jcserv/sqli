use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{
    app::{App, Mode}, 
    navigation::{PaneId, Navigable},
    panes::{
        collections::CollectionsPane, 
        header::HeaderPane,
        results::ResultsPane, 
        workspace::WorkspacePane, 
        traits::Instructions
    },
};

pub struct UI {
    header_pane: HeaderPane,
    collections_pane: CollectionsPane,
    workspace_pane: WorkspacePane,
    results_pane: ResultsPane,
}

impl UI {
    pub fn new() -> Self {
        Self {
            header_pane: HeaderPane::new(),
            collections_pane: CollectionsPane::new(),
            workspace_pane: WorkspacePane::new(),
            results_pane: ResultsPane::new(),
        }
    }

    pub fn update_dimensions(&self, app: &mut App, height: u16) {
        self.workspace_pane.update_dimensions(app, height);
    }

    pub fn render(&mut self, app: &mut App, frame: &mut Frame) {
        let search_height = if app.search.open { 3 } else { 0 };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),     
                Constraint::Min(0),        
                Constraint::Length(3),    
            ])
            .split(frame.area());

        let top_bar = chunks[0];
        let main_area = chunks[1];
        let status_area = chunks[2];

        self.header_pane.render(app, frame, top_bar);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),    // Collections tree
                Constraint::Percentage(80),    // Right side
            ])
            .split(main_area);

        let left_panel = main_chunks[0];
        let right_panel = main_chunks[1];

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70),    // Workspace
                Constraint::Percentage(30),    // Results
            ])
            .split(right_panel);

        let workspace_area = right_chunks[0];
        let results_area = right_chunks[1];

        self.collections_pane.render(app, frame, left_panel);
        self.workspace_pane.render(app, frame, workspace_area, search_height);
        self.results_pane.render(app, frame, results_area);
        self.render_instructions(app, frame, status_area);
    }

    pub fn render_instructions(&self, app: &App, frame: &mut Frame, area: Rect) {
        let instructions = match app.mode {
            Mode::Normal => {
                if let Some(active_pane) = app.navigation.active_pane() {
                    match active_pane {
                        PaneId::Header => self.header_pane.get_instructions(app),
                        PaneId::Collections => self.collections_pane.get_instructions(app),
                        PaneId::Workspace => self.workspace_pane.get_instructions(app),
                        PaneId::Results => self.results_pane.get_instructions(app),
                    }
                } else {
                    Line::from("")
                }
            },
            Mode::Command => Line::from(vec![
                " ESC ".blue().bold(),
                "Normal ".white().into(),
                " Enter ".blue().bold(),
                "Execute ".white().into(),
                " ^C ".blue().bold(),
                "Quit ".white().into(),
            ]),
            Mode::Search => self.workspace_pane.get_instructions(app),
        };
        
        let status = Paragraph::new(instructions)
            .style(Style::default().fg(Color::LightBlue))
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(status, area);
    }

    pub fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        match app.navigation.active_pane().unwrap() {
            PaneId::Header => self.header_pane.handle_key_event(app, key_event),
            PaneId::Collections => self.collections_pane.handle_key_event(app, key_event),
            PaneId::Workspace => self.workspace_pane.handle_key_event(app, key_event),
            PaneId::Results => self.results_pane.handle_key_event(app, key_event),
        }
    }

    pub fn handle_mouse_event(&self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        use crossterm::event::{MouseEventKind, MouseButton};
        
        if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
            let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
            let width = terminal_size.0 as usize;
            let height = terminal_size.1 as usize;
            
            let x = mouse_event.column as usize;
            let y = mouse_event.row as usize;
            
            if y > 1 && y < height - 3 {
                let content_height = height - 5;
                
                if x < width * 15 / 100 {
                    return self.collections_pane.handle_mouse_event(app, mouse_event);
                } else if y < 1 + (content_height * 70 / 100) {
                    return self.workspace_pane.handle_mouse_event(app, mouse_event);
                } else {
                    return self.results_pane.handle_mouse_event(app, mouse_event);
                }
            }
        }

        Ok(false)
    }
}