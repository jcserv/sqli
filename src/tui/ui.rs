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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),     
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
                Constraint::Percentage(10),    // Collections tree
                Constraint::Percentage(90),    // Right side
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

        self.render_collections(app, frame, left_panel);
        self.render_workspace(app, frame, workspace_area, search_height);
        self.render_result(app, frame, results_area);
        self.render_instructions(app, frame, status_area);
    }

    fn render_app_info(&self, app: &App, frame: &mut Frame, area: Rect) {
        let app_info_line = Line::from(vec![
            " sqli ".white().bold(),
            "v0.1.0 ".white().into(),
        ]);

        let titles = vec!["Collections", "Workspace", "Result"]
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let index = match i {
                    0 => Tab::Collections,
                    1 => Tab::Workspace,
                    _ => Tab::Result,
                };
                
                if app.current_tab == index {
                    Span::styled(*t, Style::default().fg(Color::White))
                } else {
                    Span::styled(*t, Style::default().fg(Color::Gray))
                }
            })
            .collect::<Vec<_>>();

        let tabs = Tabs::new(titles)
            .block(Block::default())
            .divider("|")
            .padding("  ", "  ");

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(area);

        let title = Paragraph::new(app_info_line)
            .style(Style::default());

        frame.render_widget(title, chunks[0]);
        frame.render_widget(tabs, chunks[1]);
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

    fn render_workspace(&self, app: &App, frame: &mut Frame, area: Rect, search_height: u16) {
        let block = Block::default()
            .title("Workspace").white().bold()
            .borders(Borders::ALL);

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

    fn render_result(&self, _app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Results").white().bold()
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