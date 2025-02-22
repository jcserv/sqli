use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent, KeyCode};
use ratatui::{
    prelude::*, 
    Frame,
    layout::{Constraint, Rect}, 
    style::{Color, Modifier, Style}, 
    text::Line, 
    widgets::{Block, Borders, Cell, Row, Scrollbar, ScrollbarState, ScrollbarOrientation, Table, TableState}, 
};

use crate::{sql::interface::QueryResult, tui::app::{App, Mode}};
use crate::tui::navigation::{Navigable, PaneId, FocusType};
use super::traits::Instructions;

pub struct ResultsPane {
    table_state: TableState,
}

impl ResultsPane {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default().with_selected(0),
        }
    }

    pub fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let focus_type = if let Some(info) = app.navigation.get_pane_info(PaneId::Results) {
            info.focus_type
        } else {
            FocusType::Inactive
        };
    
        let focus_style = match focus_type {
            FocusType::Editing => Style::default().fg(Color::LightBlue).bold(),
            FocusType::Active => Style::default().fg(Color::LightBlue),
            FocusType::Inactive => Style::default().fg(Color::White),
        };

        let execution_time_ms = app.query_result.execution_time.as_millis();    
        let status_text = format!(
            " Query time: {}ms | {} rows ", 
            execution_time_ms,
            app.query_result.row_count, 
        );
        let status_line = Line::from(status_text);
    
        let block = Block::default()
            .title_top("Results")
            .title_alignment(Alignment::Left)
            .title_style(focus_style)
            .title_bottom(status_line)
            // .title_alignment(Alignment::Right)
            .borders(Borders::ALL)
            .border_style(focus_style);
    
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        
        let header_cells = app.query_result.columns.iter()
            .map(|h| Cell::from(h.as_str()).style(Style::default().bold()));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);
    
        let constraints = calculate_column_constraints(&app.query_result);
        
        let row_count = app.query_result.rows.len();
            
        let rows = app.query_result.rows.iter().map(|item| {
            let cells = item.iter().map(|c| Cell::from(c.as_str()));
            Row::new(cells).height(1)
        });
        
        let table = Table::new(
            rows,
            constraints,
        )
        .header(header)
        .row_highlight_style(
            Style::default()
            .bg(Color::LightBlue)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol(">> ");
    
        frame.render_stateful_widget(table, inner_area, &mut self.table_state);
    
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
    
        frame.render_stateful_widget(
            scrollbar,
            inner_area,
            &mut ScrollbarState::new(
                row_count
            ).position(self.table_state.selected().unwrap_or(0)),
        );
    }

    pub fn next_row(&mut self, max_idx: usize) {
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

    pub fn previous_row(&mut self, max_idx: usize) {
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

impl Instructions for ResultsPane {
    fn get_instructions(&self, app: &App) -> Line<'static> {
        if app.mode != Mode::Normal || !app.navigation.is_active(PaneId::Results) {
            return Line::from("");
        }
        
        let is_editing = app.is_pane_in_edit_mode(PaneId::Results);
        if is_editing {
            Line::from(vec![
                " Esc ".blue().bold(),
                "Stop Editing ".white().into(),
                " ↑/↓ ".blue().bold(),
                "Navigate ".white().into(),
                " ^P ".blue().bold(),
                "Command ".white().into(),
                " ^C ".blue().bold(),
                "Quit ".white().into(),
            ])
        } else {
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
}

impl Navigable for ResultsPane {
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        if app.mode != Mode::Normal || !app.navigation.is_active(PaneId::Results) {
            return Ok(false);
        }

        let info = app.navigation.get_pane_info(PaneId::Results).unwrap();
        match info.focus_type {
            FocusType::Active => {
                match key_event.code {
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
            },
            FocusType::Editing => {
                match key_event.code {
                    KeyCode::Esc  => {
                        self.deactivate(app)
                    },
                    KeyCode::Up => {
                        self.previous_row(app.query_result.rows.len().saturating_sub(1));
                        Ok(false)
                    },
                    KeyCode::Down => {
                        self.next_row(app.query_result.rows.len().saturating_sub(1));
                        Ok(false)
                    },
                    _ => Ok(false)
                }
            },
            _ => Ok(false)
        }
    }
    
    fn handle_mouse_event(&self, app: &mut App, _mouse_event: MouseEvent) -> Result<bool> {
        if app.navigation.is_active(PaneId::Results) {
            return self.activate(app)
        }
        app.navigation.activate_pane(PaneId::Results)?;
        Ok(false)
    }
    
    fn activate(&self, app: &mut App) -> Result<bool> {
        app.navigation.start_editing(PaneId::Results)?;
        Ok(false)
    }
    
    fn deactivate(&self, app: &mut App) -> Result<bool> {
        app.navigation.stop_editing(PaneId::Results)?;
        Ok(false)
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