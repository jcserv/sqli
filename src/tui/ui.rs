use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{
    app::{App, Focus, Mode},
    collections_pane::CollectionsPane,
    results_pane::ResultsPane,
    workspace_pane::WorkspacePane,
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
                Constraint::Percentage(15),    // Collections tree
                Constraint::Percentage(85),    // Right side
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

    fn render_instructions(&self, app: &mut App, frame: &mut Frame, area: Rect) {
        let instructions = match app.mode {
            Mode::Normal => {
                match app.focus {
                    Focus::Collections => {
                        Line::from(vec![
                            " ↑/↓ ".blue().bold(),
                            "Navigate ".white().into(),
                            " ←/→ ".blue().bold(),
                            "Collapse/Expand ".white().into(),
                            " Space ".blue().bold(),
                            "Select ".white().into(),
                            " Tab ".blue().bold(),
                            "Switch Panel ".white().into(),
                            " ^P ".blue().bold(),
                            "Command ".white().into(),
                            " ^C ".blue().bold(),
                            "Quit ".white().into(),
                        ])
                    },
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
                    Focus::Result => {
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
            },
            Mode::Command => Line::from(vec![
                " ESC ".blue().bold(),
                "Normal ".white().into(),
                " Enter ".blue().bold(),
                "Execute ".white().into(),
                " ^C ".blue().bold(),
                "Quit ".white().into(),
            ]),
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
            }
        };
        
        let status = Paragraph::new(instructions)
            .style(Style::default().fg(Color::LightBlue))
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(status, area);
    }
    
    pub fn update_dimensions(&self, app: &mut App, height: u16) {
        self.workspace_pane.update_dimensions(app, height);
    }
}