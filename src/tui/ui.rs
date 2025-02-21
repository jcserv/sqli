use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{
    app::{App, Focus, Mode}, panes::{collections::CollectionsPane, results::ResultsPane, workspace::WorkspacePane}, traits::{Instructions, PaneEventHandler}
};

pub struct UI {
    collections_pane: CollectionsPane,
    workspace_pane: WorkspacePane,
    results_pane: ResultsPane,
}

impl UI {
    pub fn new() -> Self {
        Self {
            collections_pane: CollectionsPane::new(),
            workspace_pane: WorkspacePane::new(),
            results_pane: ResultsPane::new(),
        }
    }

    pub fn update_dimensions(&self, app: &mut App, height: u16) {
        self.workspace_pane.update_dimensions(app, height);
    }

    pub fn render(&self, app: &mut App, frame: &mut Frame) {
        let search_height = if app.search.open { 3 } else { 0 };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),     
                Constraint::Min(0),        
                Constraint::Length(3),    
            ])
            .split(frame.area());

        let top_bar = chunks[0];
        let main_area = chunks[1];
        let status_area = chunks[2];

        self.render_app_info(app, frame, top_bar);

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

    fn render_app_info(&self, _app: &App, frame: &mut Frame, area: Rect) {
        let app_info_line = Line::from(vec![
            " sqli ".white().bold(),
            "v0.1.0 ".white().into(),
        ]);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12),
                Constraint::Min(0),
            ])
            .split(area);

        let title = Paragraph::new(app_info_line)
            .style(Style::default());

        frame.render_widget(title, chunks[0]);
    }

    pub fn render_instructions(&self, app: &App, frame: &mut Frame, area: Rect) {
        let instructions = match app.mode {
            Mode::Normal => {
                match app.focus {
                    Focus::Collections | Focus::CollectionsEdit => self.collections_pane.get_instructions(app),
                    Focus::Workspace | Focus::WorkspaceEdit => self.workspace_pane.get_instructions(app),
                    Focus::Result => self.results_pane.get_instructions(app),
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