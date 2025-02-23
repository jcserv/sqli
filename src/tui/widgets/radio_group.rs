use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Layout, Direction, Constraint, Position},
    style::{Color, Style},
    widgets::Widget,
};

#[derive(Debug, Clone)]
pub struct RadioOption<'a> {
    pub label: &'a str,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct RadioGroup<'a> {
    options: Vec<RadioOption<'a>>,
    selected: usize,
    style: Style,
    highlight_style: Style,
}

impl<'a> RadioGroup<'a> {
    pub fn new(options: Vec<RadioOption<'a>>) -> Self {
        Self {
            options,
            selected: 0,
            style: Style::default(),
            highlight_style: Style::default().fg(Color::Blue),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn set_selected(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected = index;
        }
    }

    pub fn selected_value(&self) -> String {
        self.options[self.selected].value.clone()
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    pub fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.options.len() - 1
        } else {
            self.selected - 1
        };
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left | KeyCode::Up => {
                self.previous();
                true
            }
            KeyCode::Right | KeyCode::Down => {
                self.next();
                true
            }
            _ => false
        }
    }

    pub fn handle_mouse_event(&mut self, mouse: MouseEvent, area: Rect) -> bool {
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let position = Position::new(mouse.column, mouse.row);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    std::iter::repeat(Constraint::Ratio(1, self.options.len().try_into().unwrap()))
                        .take(self.options.len())
                        .collect::<Vec<_>>()
                )
                .split(area);

            for (i, chunk) in chunks.iter().enumerate() {
                if chunk.contains(position) {
                    self.selected = i;
                    return true;
                }
            }
        }
        false
    }
}

impl<'a> Widget for RadioGroup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                std::iter::repeat(Constraint::Ratio(1, self.options.len().try_into().unwrap()))
                    .take(self.options.len())
                    .collect::<Vec<_>>()
            )
            .split(area);

        for (i, option) in self.options.iter().enumerate() {
            let style = if i == self.selected {
                self.highlight_style
            } else {
                self.style
            };

            let symbol = if i == self.selected { "(*)" } else { "( )" };
            let text = format!("{} {}", symbol, option.label);
            
            buf.set_string(
                chunks[i].x,
                chunks[i].y,
                text,
                style,
            );
        }
    }
}