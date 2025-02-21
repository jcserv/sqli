use anyhow::{Ok, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, prelude::*, text::Line, widgets::{Block, Borders, Paragraph}, Frame
};

use super::{
    app::{App, Focus, Mode},
    panes::{
        collections::CollectionsPane,
        results::ResultsPane,
        header::HeaderPane,
        traits::{Instructions, PaneEventHandler},
        workspace::WorkspacePane,
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
                Constraint::Length(4),     // Header area - fixed height
                Constraint::Min(0),        // Main content area
                Constraint::Length(3),     // Status bar - fixed height
            ])
            .split(frame.area());

        let header = chunks[0];
        let main_area = chunks[1];
        let status_area = chunks[2];

        self.header_pane.render(app, frame, header);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),    // Collections tree
                Constraint::Percentage(80),    // Right side
            ])
            .split(main_area);

        let left_panel = main_chunks[0];
        let right_panel = main_chunks[1];

        // Split right panel into workspace and results
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70),    // Workspace
                Constraint::Percentage(30),    // Results
            ])
            .split(right_panel);

        let workspace_area = right_chunks[0];
        let results_area = right_chunks[1];

        // Render panes
        self.collections_pane.render(app, frame, left_panel);
        self.workspace_pane.render(app, frame, workspace_area, search_height);
        self.results_pane.render(app, frame, results_area);
        
        // Render instructions
        self.render_instructions(app, frame, status_area);
    }

    pub fn render_instructions(&self, app: &App, frame: &mut Frame, area: Rect) {
        let instructions = match app.mode {
            Mode::Normal => {
                match app.focus {
                    Focus::Collections | Focus::CollectionsEdit => self.collections_pane.get_instructions(app),
                    Focus::Workspace | Focus::WorkspaceEdit => self.workspace_pane.get_instructions(app),
                    Focus::Result => self.results_pane.get_instructions(app),
                    Focus::Header => self.header_pane.get_instructions(app),
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
        // Handle global key events first
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                app.should_quit = true;
                return Ok(true);
            }
            (KeyCode::Tab, _) if app.focus != Focus::WorkspaceEdit => {
                app.cycle_tab();
                return Ok(false);
            }
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                app.mode = Mode::Command;
                return Ok(false);
            }
            _ => {}
        }
        
        match app.focus {
            Focus::Collections | Focus::CollectionsEdit => self.collections_pane.handle_key_event(app, key_event),
            Focus::Workspace | Focus::WorkspaceEdit => self.workspace_pane.handle_key_event(app, key_event),
            Focus::Result => self.results_pane.handle_key_event(app, key_event),
            Focus::Header => self.header_pane.handle_key_event(app, key_event),
        }
    }

    pub fn handle_mouse_event(&self, app: &mut App, mouse_event: MouseEvent) -> Result<bool> {
        use crossterm::event::{MouseEventKind, MouseButton};
        
        if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
            let y = mouse_event.row as usize;
            
            // Header area (0-2)
            if y < 3 {
                return self.header_pane.handle_mouse_event(app, mouse_event);
            }
            
            // Status bar area (bottom 3 lines)
            let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
            let height = terminal_size.1 as usize;
            if y >= height - 3 {
                return Ok(false); // Status bar doesn't handle events
            }
            
            let content_height = height - 6; // Excluding header and status bar
            let x = mouse_event.column as usize;
            let width = terminal_size.0 as usize;
            
            if x < width * 20 / 100 {
                return self.collections_pane.handle_mouse_event(app, mouse_event);
            } else if y < 3 + (content_height * 70 / 100) {
                return self.workspace_pane.handle_mouse_event(app, mouse_event);
            } else {
                return self.results_pane.handle_mouse_event(app, mouse_event);
            }
        }
        Ok(false)
    }
}