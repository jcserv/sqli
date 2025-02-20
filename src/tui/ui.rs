use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use super::app::{App, Mode, Tab};

pub struct UI;

impl UI {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, app: &App, frame: &mut Frame) {
        let search_height = if app.search.open { 3 } else { 0 };

        let outer_padding = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1),    // Left padding
                Constraint::Min(0),       // Content
                Constraint::Length(1),    // Right padding
            ])
            .split(frame.area());

        let vertical_padding = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),    // Top padding
                Constraint::Min(0),       // Content
                Constraint::Length(1),    // Bottom padding
            ])
            .split(outer_padding[1]);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),                // App info
                Constraint::Length(3),                // Tabs
                Constraint::Length(search_height),    // Search box
                Constraint::Min(0),                   // Main content
                Constraint::Length(3),                // Status line
            ])
            .split(vertical_padding[1]);

        self.render_app_info(app, frame, chunks[0]);
        self.render_tabs(app, frame, chunks[1]);
        
        if app.search.open {
            frame.render_widget(&app.search.textarea, chunks[2]);
        }
        
        self.render_main_content(app, frame, chunks[3]);
        self.render_instructions(app, frame, chunks[4]);
    }

    fn render_app_info(&self, _app: &App, frame: &mut Frame, area: Rect) {
        let line: Line = vec![
            " sqli".white().bold(),
            " ".into(),
            "v0.1.0".white().into(),
        ].into();

        let block = Block::default()
            .title(line);

        frame.render_widget(block, area);
    }

    fn render_tabs(&self, app: &App, frame: &mut Frame, area: Rect) {
        let titles = vec!["Collections", "Workspace", "Result"]
            .iter()
            .map(|t| Span::styled(*t, Style::default().fg(Color::LightBlue)))
            .collect::<Vec<_>>();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(match app.current_tab {
                Tab::Collections => 0,
                Tab::Workspace => 1,
                Tab::Result => 2,
            })
            .style(Style::default().fg(Color::Blue))
            .highlight_style(Style::default().fg(Color::White));

        frame.render_widget(tabs, area);
    }

    fn render_main_content(&self, app: &App, frame: &mut Frame, area: Rect) {
        match app.current_tab {
            Tab::Collections => self.render_collections(app, frame, area),
            Tab::Workspace => frame.render_widget(&app.workspace, area),
            Tab::Result => self.render_result(app, frame, area),
        }
    }

    fn render_collections(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Collections").white().bold()
            .borders(Borders::ALL);

        let collections = app.collections
            .keys()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let paragraph = Paragraph::new(collections)
            .block(block)
            .style(Style::default().fg(Color::LightBlue));

        frame.render_widget(paragraph, area);
    }

    fn render_result(&self, _app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Result").white().bold()
            .borders(Borders::ALL);

        frame.render_widget(block, area);
    }

    fn render_instructions(&self, app: &App, frame: &mut Frame, area: Rect) {
        let instructions = match app.mode {
            Mode::Normal => {
                if app.current_tab == Tab::Workspace {
                    Line::from(vec![
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
                } else {
                    Line::from(vec![
                        " ^P ".blue().bold(),
                        "Command ".white().into(),
                        " ^C ".blue().bold(),
                        "Quit ".white().into(),
                    ])
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
}