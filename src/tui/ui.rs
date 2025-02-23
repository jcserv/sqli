use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{
    app::{App, Mode}, 
    navigation::PaneId,
    panes::{
        collections::CollectionsPane, header::HeaderPane, pane::PaneExt, results::ResultsPane, workspace::WorkspacePane 
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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),     
                Constraint::Min(0),        
                Constraint::Length(3),     // Instructions area  
                Constraint::Length(1),     // Status message area
            ])
            .split(frame.area());

        let top_bar = chunks[0];
        let main_area = chunks[1];
        let instructions_area = chunks[2];
        let status_area = chunks[3];

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
        self.workspace_pane.render(app, frame, workspace_area);
        self.results_pane.render(app, frame, results_area);
        self.render_status_message(app, frame, status_area);
        self.render_instructions(app, frame, instructions_area);

        if app.modal_manager.is_modal_active() {
            let area = frame.area();
            app.modal_manager.render(frame, area);
        }
    }

    fn render_status_message(&self, app: &App, frame: &mut Frame, area: Rect) {
        let message_style = if app.ui_state.message.starts_with("Error") {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        
        let message = Paragraph::new(app.ui_state.message.as_str())
            .style(message_style);
            
        frame.render_widget(message, area);
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
            _ => Line::from(""),
        };
        
        let status = Paragraph::new(instructions)
            .style(Style::default().fg(Color::LightBlue))
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(status, area);
    }

    pub fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        match app.navigation.active_pane().unwrap() {
            PaneId::Header => self.header_pane.handle_key_event(app, key_event),
            PaneId::Collections => self.collections_pane.handle_key_event(app, key_event),
            PaneId::Workspace => self.workspace_pane.handle_key_event(app, key_event),
            PaneId::Results => self.results_pane.handle_key_event(app, key_event),
        }
    }

    pub fn handle_mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {        
        let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
        let width = terminal_size.0 as usize;
        let height = terminal_size.1 as usize;
        
        let x = mouse_event.column as usize;
        let y = mouse_event.row as usize;
        
        if y < 4 {
            return self.header_pane.handle_mouse_event(app, mouse_event);
        }
        
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
        Ok(false)
    }
}