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

        // Main content area within the padding
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // App info
                Constraint::Length(3),  // Tabs
                Constraint::Min(0),     // Main content
                Constraint::Length(3),  // Status line
            ])
            .split(vertical_padding[1]);

        self.render_app_info(app, frame, chunks[0]);
        self.render_tabs(app, frame, chunks[1]);
        self.render_main_content(app, frame, chunks[2]);
        self.render_instructions(app, frame, chunks[3]);
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
            Tab::Workspace => self.render_workspace(app, frame, area),
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

    fn render_workspace(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Workspace").white().bold()
            .borders(Borders::ALL);

        let paragraph = Paragraph::new(app.query.clone())
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
        let instructions = Line::from(vec![
            " ^c ".blue().bold(),
            "Quit ".white().into(),
        ]);

        let mode = format!(
            "Mode: {}",
            match app.mode {
                Mode::Normal => "Normal",
                Mode::Command => "Command",
            }
        );

        let status = Paragraph::new(mode)
            .style(Style::default().fg(Color::LightBlue))
            .block(Block::default().borders(Borders::ALL).title_bottom(instructions.centered()));

        frame.render_widget(status, area);
    }

    // fn center_horizontal(&self, area: Rect, width: u16) -> Rect {
    //     let [area] = Layout::horizontal([Constraint::Length(width)])
    //         .flex(Flex::Center)
    //         .areas(area);
    //     area
    // }

    // fn center_vertical(&self, area: Rect, height: u16) -> Rect {
    //     let [area] = Layout::vertical([Constraint::Length(height)])
    //         .flex(Flex::Center)
    //         .areas(area);
    //     area
    // }
}