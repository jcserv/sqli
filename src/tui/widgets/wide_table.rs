use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, Widget},
};

use crate::sql::result::QueryResult;

pub struct WideTableState {
    pub horizontal_scroll: ScrollbarState,
    pub table_state: ratatui::widgets::TableState,
    pub scroll_position: usize,
    pub column_count: usize,
    pub visible_columns: usize,
}

impl WideTableState {
    pub fn new(column_count: usize) -> Self {
        Self {
            horizontal_scroll: ScrollbarState::new(column_count),
            table_state: ratatui::widgets::TableState::default().with_selected(0),
            scroll_position: 0,
            column_count,
            visible_columns: 0,
        }
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

    pub fn scroll_right(&mut self) {
        if self.scroll_position + self.visible_columns < self.column_count {
            self.scroll_position += 1;
            self.horizontal_scroll.next();
        }
    }

    pub fn scroll_left(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
            self.horizontal_scroll.prev();
        }
    }

    pub fn update_visible_columns(&mut self, width: u16) {
        let approx_column_width = 15;
        let max_visible = (width as usize / approx_column_width).max(1);
        self.visible_columns = max_visible.min(self.column_count);
        
        if self.scroll_position + self.visible_columns > self.column_count {
            self.scroll_position = self.column_count.saturating_sub(self.visible_columns);
        }
    }
}

pub struct WideTable<'a> {
    block: Option<Block<'a>>,
    highlight_style: Style,
    highlight_symbol: Option<&'a str>,
    widths: Vec<Constraint>,
    column_spacing: u16,
    header_style: Style,
    result: &'a QueryResult,
}

impl<'a> WideTable<'a> {
    pub fn new(result: &'a QueryResult) -> Self {
        Self {
            block: None,
            highlight_style: Style::default().add_modifier(Modifier::REVERSED),
            highlight_symbol: None,
            widths: vec![],
            column_spacing: 1,
            header_style: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            result,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn highlight_symbol(mut self, symbol: &'a str) -> Self {
        self.highlight_symbol = Some(symbol);
        self
    }

    pub fn widths(mut self, widths: Vec<Constraint>) -> Self {
        self.widths = widths;
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    fn get_visible_columns_data(&self, state: &WideTableState) -> (Vec<String>, Vec<Vec<String>>) {
        let start = state.scroll_position;
        let end = (start + state.visible_columns).min(self.result.columns.len());

        let visible_columns = self.result.columns[start..end].to_vec();
        
        let visible_rows: Vec<Vec<String>> = self.result.rows
            .iter()
            .map(|row| {
                row[start..end].to_vec()
            })
            .collect();

        (visible_columns, visible_rows)
    }

    fn calculate_widths(&self, visible_columns: &[String], visible_rows: &[Vec<String>]) -> Vec<Constraint> {
        if self.widths.len() >= visible_columns.len() {
            return self.widths[0..visible_columns.len()].to_vec();
        }

        let mut max_widths: Vec<usize> = visible_columns.iter()
            .map(|col| col.len())
            .collect();
        
        for row in visible_rows {
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

    fn render_horizontal_scrollbar(&self, area: Rect, buf: &mut Buffer, state: &mut WideTableState) {
        let scrollbar_area = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(1),
            area.width,
            1
        );

        let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .begin_symbol(None)
            .track_symbol(Some("─"))
            .thumb_symbol("█");

        StatefulWidget::render(scrollbar, scrollbar_area, buf, &mut state.horizontal_scroll);
    }
}

impl StatefulWidget for WideTable<'_> {
    type State = WideTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = match self.block {
            Some(ref b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3), // Table area
                Constraint::Length(1), // Scrollbar area
            ])
            .split(area);

        state.update_visible_columns(chunks[0].width);

        let (visible_columns, visible_rows) = self.get_visible_columns_data(state);        
        let widths = self.calculate_widths(&visible_columns, &visible_rows);

        let header_cells = visible_columns.iter()
            .map(|h| Cell::from(h.as_str()).style(self.header_style));
        let header = Row::new(header_cells)
            .height(1);

        let rows = visible_rows.iter().map(|item| {
            let cells = item.iter().map(|c| Cell::from(c.as_str()));
            Row::new(cells).height(1)
        });

        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(self.highlight_style)
            .highlight_symbol(self.highlight_symbol.unwrap());

        StatefulWidget::render(table, chunks[0], buf, &mut state.table_state);

        self.render_horizontal_scrollbar(chunks[0], buf, state);
    }
}