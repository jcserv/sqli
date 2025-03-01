use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState},
    Frame,
};

use crate::{
    sql::result::QueryResult, 
    tui::{
        app::App,
        navigation::PaneId,
    }
};

use super::pane::{Pane, PaneExt};

pub struct ResultsPane {
    table_state: TableState,
}

impl ResultsPane {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default().with_selected(0),
        }
    }

    fn next_row(&mut self, max_idx: usize) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= max_idx {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous_row(&mut self, max_idx: usize) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    max_idx
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

impl Pane for ResultsPane {
    fn pane_id(&self) -> PaneId {
        PaneId::Results
    }

    fn title(&self) -> &'static str {
        "Results"
    }

    fn title_bottom(&self, app: &App) -> String {
        let execution_time_ms = app.query_state.query_result.execution_time.as_millis();
        let status_text = format!(
            "Query time: {}ms | {} rows",
            execution_time_ms,
            app.query_state.query_result.row_count,
        );
        status_text
    }

    fn render_content(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let header_cells = app.query_state.query_result.columns.iter()
            .map(|h| Cell::from(h.as_str()).style(Style::default().bold()));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);
    
        let constraints = calculate_column_constraints(&app.query_state.query_result);
        let row_count = app.query_state.query_result.rows.len();

        if row_count == 0 || app.query_state.query_result.columns.is_empty() {
            let empty_message = Paragraph::new("No query results to display. Run a query using the button above.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            
            frame.render_widget(empty_message, area);
            return;
        }
            
        let rows = app.query_state.query_result.rows.iter().map(|item| {
            let cells = item.iter().map(|c| Cell::from(c.as_str()));
            Row::new(cells).height(1)
        });
        
        let table = Table::new(rows, constraints)
            .header(header)
            .row_highlight_style(
                Style::default()
                    .bg(Color::LightBlue)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol(">> ");
    
        frame.render_stateful_widget(table, area, &mut self.table_state);
    
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
    
        frame.render_stateful_widget(
            scrollbar,
            area,
            &mut ScrollbarState::new(row_count).position(self.table_state.selected().unwrap_or(0)),
        );
    }

    fn get_custom_instructions(&self, _app: &App, is_editing: bool) -> Line<'static> {
        if is_editing {
            Line::from(vec![
                " Esc ".blue().bold(),
                "Stop Editing ".white().into(),
                " ↑/↓ ".blue().bold(),
                "Navigate ".white().into(),
                " ^C ".blue().bold(),
                "Quit ".white().into(),
            ])
        } else {
            Line::from(vec![
                " Tab ".blue().bold(),
                "Switch Panel ".white().into(),
                " Space ".blue().bold(),
                "Select ".white().into(),
                " ^C ".blue().bold(),
                "Quit ".white().into(),
            ])
        }
    }

    fn handle_edit_mode_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.deactivate(app)
            },
            KeyCode::Up => {
                self.previous_row(app.query_state.query_result.rows.len().saturating_sub(1));
                Ok(false)
            },
            KeyCode::Down => {
                self.next_row(app.query_state.query_result.rows.len().saturating_sub(1));
                Ok(false)
            },
            _ => Ok(false)
        }
    }

    fn handle_active_mode_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate(app)
            },
            KeyCode::Up => {
                app.navigation.activate_pane(PaneId::Workspace)?;
                Ok(false)
            },
            KeyCode::Left => {
                app.navigation.activate_pane(PaneId::Collections)?;
                Ok(false)
            },
            _ => Ok(false)
        }
    }
}

fn calculate_column_constraints(query_result: &QueryResult) -> Vec<Constraint> {
    if query_result.columns.is_empty() {
        return vec![];
    }

    let mut max_widths: Vec<usize> = query_result.columns.iter()
        .map(|col| col.len())
        .collect();
    
    for row in &query_result.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < max_widths.len() {
                max_widths[i] = max_widths[i].max(cell.len());
            }
        }
    }

    max_widths.iter().map(|&width| {
        let column_width = width + 2;
        if column_width < 10 {
            Constraint::Length(column_width as u16)
        } else {
            Constraint::Min(column_width as u16)
        }
    }).collect()
}