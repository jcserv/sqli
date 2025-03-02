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
        navigation::PaneId, widgets::wide_table::{WideTable, WideTableState},
    }
};

use super::pane::{Pane, PaneExt};

pub struct ResultsPane {
    table_state: TableState,
    wide_table_state: Option<WideTableState>,
    use_wide_table: bool,
}

impl Default for ResultsPane {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultsPane {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default().with_selected(0),
            wide_table_state: None,
            use_wide_table: false,
        }
    }

    fn next_row(&mut self, max_idx: usize) {
        if self.use_wide_table {
            if let Some(state) = &mut self.wide_table_state {
                state.next_row(max_idx);
            }
        } else {
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
    }

    fn previous_row(&mut self, max_idx: usize) {
        if self.use_wide_table {
            if let Some(state) = &mut self.wide_table_state {
                state.previous_row(max_idx);
            }
        } else {
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

    fn scroll_left(&mut self) {
        if let Some(state) = &mut self.wide_table_state {
            state.scroll_left();
        }
    }

    fn scroll_right(&mut self) {
        if let Some(state) = &mut self.wide_table_state {
            state.scroll_right();
        }
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
        // let header_cells = app.query_state.query_result.columns.iter()
        //     .map(|h| Cell::from(h.as_str()).style(Style::default().bold()));
        // let header = Row::new(header_cells)
        //     .style(Style::default().bg(Color::DarkGray))
        //     .height(1);
    
        // let constraints = calculate_column_constraints(&app.query_state.query_result);
        let column_count = app.query_state.query_result.columns.len();
        let row_count = app.query_state.query_result.rows.len();

        if row_count == 0 || app.query_state.query_result.columns.is_empty() {
            let empty_message = Paragraph::new("No query results to display. Run a query using the button above.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            
            frame.render_widget(empty_message, area);
            return;
        }

        self.use_wide_table = column_count > 8;
            
        if self.use_wide_table {
            if self.wide_table_state.is_none() {
                self.wide_table_state = Some(WideTableState::new(column_count));
            }

            if let Some(state) = &mut self.wide_table_state {
                let wide_table = WideTable::new(&app.query_state.query_result)
                    .highlight_style(
                        Style::default()
                            .bg(Color::LightBlue)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD)
                    )
                    .highlight_symbol(">> ");

                frame.render_stateful_widget(wide_table, area, state);
            }
        } else {
            let header_cells = app.query_state.query_result.columns.iter()
                .map(|h| Cell::from(h.as_str()).style(Style::default().bold()));
            let header = Row::new(header_cells)
                .style(Style::default().bg(Color::DarkGray))
                .height(1);
        
            let constraints = calculate_column_constraints(&app.query_state.query_result);
            
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
    }

    fn get_custom_instructions(&self, _app: &App, is_editing: bool) -> Line<'static> {
        if is_editing {
            if self.use_wide_table {
                Line::from(vec![
                    " Esc ".blue().bold(),
                    "Stop Editing ".white(),
                    " ↑/↓ ".blue().bold(),
                    "Navigate Rows ".white(),
                    " ←/→ ".blue().bold(),
                    "Scroll Columns ".white(),
                    " ^C ".blue().bold(),
                    "Quit ".white(),
                ])
            } else {
                Line::from(vec![
                    " Esc ".blue().bold(),
                    "Stop Editing ".white(),
                    " ↑/↓ ".blue().bold(),
                    "Navigate ".white(),
                    " ^C ".blue().bold(),
                    "Quit ".white(),
                ])
            }
        } else {
            Line::from(vec![
                " Tab ".blue().bold(),
                "Switch Panel ".white(),
                " Space ".blue().bold(),
                "Select ".white(),
                " ^C ".blue().bold(),
                "Quit ".white(),
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
            KeyCode::Left if self.use_wide_table => {
                self.scroll_left();
                Ok(false)
            },
            KeyCode::Right if self.use_wide_table => {
                self.scroll_right();
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