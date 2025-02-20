use ratatui::{style::{Color, Style}, widgets::{Block, Borders, Widget}};
use std::ops::{Deref, DerefMut};
use tui_textarea::TextArea;

const LINE_OFFSET: i32 = 10;

pub struct SearchableTextArea<'a> {
    inner: TextArea<'a>,
    search_pattern: String,
    last_search_pos: (usize, usize), // (line, column)
    initialized_height: u16, // Track the last height we initialized with
}

impl<'a> Default for SearchableTextArea<'a> {
    fn default() -> Self {
        Self {
            inner: TextArea::default(),
            search_pattern: String::new(),
            last_search_pos: (0, 0),
            initialized_height: 0,
        }
    }
}

impl<'a> Deref for SearchableTextArea<'a> {
    type Target = TextArea<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> DerefMut for SearchableTextArea<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a> Widget for &'a SearchableTextArea<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Widget::render(&self.inner, area, buf)
    }
}

impl<'a> SearchableTextArea<'a> {
    pub fn init(&mut self) {
        self.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Workspace")
        );
        self.set_line_number_style(Style::default().bg(Color::DarkGray));
        self.set_cursor_line_style(Style::default());
    }

    pub fn update_dimensions(&mut self, height: u16) {
        if height <= self.initialized_height {
            return;
        }

        let visible_lines = (height as i32 - LINE_OFFSET) as usize;
        
        let current_lines = self.inner.lines().len();
        if visible_lines > current_lines {
            for _ in current_lines..visible_lines {
                self.inner.insert_str("\n");
            }
        }
        
        self.initialized_height = height;
        self.inner.move_cursor(tui_textarea::CursorMove::Top);
    }

    pub fn get_content(&self) -> String {
        let lines = self.inner.lines();
        let mut last_non_empty = lines.len();
        
        for (i, line) in lines.iter().enumerate().rev() {
            if !line.trim().is_empty() {
                last_non_empty = i + 1;
                break;
            }
        }
        
        lines[..last_non_empty].join("\n")
    }

    pub fn set_search_pattern(&mut self, pattern: &str) -> anyhow::Result<()> {
        self.search_pattern = pattern.to_string();
        self.last_search_pos = self.inner.cursor();
        Ok(())
    }

    pub fn search_forward(&mut self, from_start: bool) -> bool {
        if self.search_pattern.is_empty() {
            return false;
        }

        let start_line = if from_start { 0 } else { self.last_search_pos.0 };
        let mut start_col = if from_start { 0 } else { self.last_search_pos.1 + 1 };

        // Search from current position to end
        for line_idx in start_line..self.inner.lines().len() {
            let line = &self.inner.lines()[line_idx];
            
            // If we're on a new line, start from beginning
            if line_idx > start_line {
                start_col = 0;
            }

            if let Some(col_idx) = line[start_col..].find(&self.search_pattern) {
                let col_idx = col_idx + start_col;
                self.inner.move_cursor(tui_textarea::CursorMove::Jump(line_idx.try_into().unwrap(), col_idx.try_into().unwrap()));
                self.last_search_pos = (line_idx, col_idx);
                return true;
            }
        }

        // If we started from middle and didn't find anything, try from beginning
        if !from_start && start_line > 0 {
            for line_idx in 0..=self.last_search_pos.0 {
                let line = &self.inner.lines()[line_idx];
                
                let search_end = if line_idx == self.last_search_pos.0 {
                    self.last_search_pos.1
                } else {
                    line.len()
                };

                if let Some(col_idx) = line[..search_end].find(&self.search_pattern) {
                    self.inner.move_cursor(tui_textarea::CursorMove::Jump(line_idx.try_into().unwrap(), col_idx.try_into().unwrap()));
                    self.last_search_pos = (line_idx, col_idx);
                    return true;
                }
            }
        }

        false
    }

    pub fn search_back(&mut self, from_end: bool) -> bool {
        if self.search_pattern.is_empty() {
            return false;
        }

        let start_line = if from_end {
            self.inner.lines().len() - 1
        } else {
            self.last_search_pos.0
        };
        
        let mut start_col = if from_end {
            self.inner.lines()[start_line].len()
        } else {
            self.last_search_pos.1
        };

        // Search from current position to start
        for line_idx in (0..=start_line).rev() {
            let line = &self.inner.lines()[line_idx];
            
            // If we're on a previous line, start from end
            if line_idx < start_line {
                start_col = line.len();
            }

            if let Some(col_idx) = line[..start_col].rfind(&self.search_pattern) {
                self.inner.move_cursor(tui_textarea::CursorMove::Jump(line_idx.try_into().unwrap(), col_idx.try_into().unwrap()));
                self.last_search_pos = (line_idx, col_idx);
                return true;
            }
        }

        // If we started from middle and didn't find anything, try from end
        if !from_end && start_line < self.inner.lines().len() - 1 {
            for line_idx in (self.last_search_pos.0 + 1..self.inner.lines().len()).rev() {
                let line = &self.inner.lines()[line_idx];
                
                if let Some(col_idx) = line.rfind(&self.search_pattern) {
                    self.inner.move_cursor(tui_textarea::CursorMove::Jump(line_idx.try_into().unwrap(), col_idx.try_into().unwrap()));
                    self.last_search_pos = (line_idx, col_idx);
                    return true;
                }
            }
        }

        false
    }

    pub fn replace_next(&mut self, replacement: &str) -> bool {
        if self.search_pattern.is_empty() {
            return false;
        }

        let (line_idx, col_idx) = self.last_search_pos;
        if line_idx >= self.inner.lines().len() {
            return false;
        }

        let line = self.inner.lines()[line_idx].clone();
        if col_idx + self.search_pattern.len() > line.len() {
            return false;
        }

        // Check if pattern matches at current position
        if line[col_idx..].starts_with(&self.search_pattern) {
            // Create new line with replacement
            let mut new_line = line[..col_idx].to_string();
            new_line.push_str(replacement);
            new_line.push_str(&line[col_idx + self.search_pattern.len()..]);
            
            // Replace the line
            self.delete_line();
            self.inner.insert_str(&new_line);
            
            // Move cursor to end of replacement
            self.last_search_pos = (line_idx, col_idx + replacement.len());
            self.inner.move_cursor(tui_textarea::CursorMove::Jump(line_idx.try_into().unwrap(), (col_idx + replacement.len()).try_into().unwrap()));
            
            true
        } else {
            false
        }
    }

    pub fn replace_all(&mut self, replacement: &str) -> usize {
        if self.search_pattern.is_empty() {
            return 0;
        }

        let mut count = 0;
        let mut last_pos = (0, 0);

        while self.search_forward(false) {
            let current_pos = self.last_search_pos;
            if current_pos == last_pos {
                break;
            }
            if self.replace_next(replacement) {
                count += 1;
            }
            last_pos = current_pos;
        }

        count
    }

    pub fn delete_line(&mut self) {
        let (_row, _) = self.inner.cursor();
        self.inner.move_cursor(tui_textarea::CursorMove::Head);
        self.inner.move_cursor(tui_textarea::CursorMove::End);
        self.inner.delete_line_by_head();
    }
}