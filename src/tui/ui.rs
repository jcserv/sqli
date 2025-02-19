use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
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
        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tabs
                Constraint::Min(0),     // Main content
                Constraint::Length(3),  // Status line
            ])
            .split(frame.area());

        self.render_tabs(app, frame, chunks[0]);
        self.render_main_content(app, frame, chunks[1]);
        self.render_status_line(app, frame, chunks[2]);
    }

    fn render_tabs(&self, app: &App, frame: &mut Frame, area: Rect) {
        let titles = vec!["History", "Collections", "Response"]
            .iter()
            .map(|t| Span::styled(*t, Style::default().fg(Color::White)))
            .collect::<Vec<_>>();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(match app.current_tab {
                Tab::History => 0,
                Tab::Collections => 1,
                Tab::Response => 2,
            })
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(tabs, area);
    }

    fn render_main_content(&self, app: &App, frame: &mut Frame, area: Rect) {
        match app.current_tab {
            Tab::History => self.render_history(app, frame, area),
            Tab::Collections => self.render_collections(app, frame, area),
            Tab::Response => self.render_response(app, frame, area),
        }
    }

    fn render_history(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Query History")
            .borders(Borders::ALL);
        
        let queries = app.queries
            .iter()
            .map(|q| q.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let paragraph = Paragraph::new(queries)
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    fn render_collections(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Collections")
            .borders(Borders::ALL);

        let collections = app.collections
            .keys()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let paragraph = Paragraph::new(collections)
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    fn render_response(&self, _app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Response")
            .borders(Borders::ALL);

        frame.render_widget(block, area);
    }

    fn render_status_line(&self, app: &App, frame: &mut Frame, area: Rect) {
        let mode = format!(
            "Mode: {}",
            match app.mode {
                Mode::Normal => "Normal",
                Mode::Insert => "Insert",
                Mode::Command => "Command",
            }
        );

        let status = Paragraph::new(mode)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(status, area);
    }
}