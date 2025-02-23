// https://github.com/ratatui/ratatui/blob/main/examples/apps/custom-widget/src/main.rs

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Normal,
    Hover,
    Selected,
    Active,
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    text: Color,
    background: Color,
    highlight: Color,
    shadow: Color,
    hover: Color,
}

#[derive(Debug, Clone)]
pub struct Button<'a> {
    label: Line<'a>,
    theme: Theme,
    state: State,
    area: Option<Rect>,
}

pub const RED: Theme = Theme {
    text: Color::White,
    background: Color::Rgb(144, 48, 48),
    highlight: Color::Rgb(192, 64, 64),
    shadow: Color::Rgb(96, 32, 32),
    hover: Color::Rgb(168, 64, 64),
};

pub const GREEN: Theme = Theme {
    text: Color::White,
    background: Color::Rgb(48, 144, 48),
    highlight: Color::Rgb(64, 192, 64),
    shadow: Color::Rgb(32, 96, 32),
    hover: Color::Rgb(64, 168, 64),
};

pub const BLUE: Theme = Theme {
    text: Color::White,
    background: Color::Rgb(48, 72, 144),
    highlight: Color::Rgb(64, 96, 192),
    shadow: Color::Rgb(32, 48, 96),
    hover: Color::Rgb(64, 88, 168),
};

/// A button with a label that can be themed and supports hover state.
impl<'a> Button<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Button {
            label: label.into(),
            theme: BLUE,
            state: State::Normal,
            area: None,
        }
    }

    pub const fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area = Some(area);
    }

    pub fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> bool {
        match mouse_event.kind {
            MouseEventKind::Moved => {
                if let Some(area) = self.area {
                    let is_over_button = 
                        mouse_event.row >= area.y && 
                        mouse_event.row < area.y + area.height &&
                        mouse_event.column >= area.x && 
                        mouse_event.column < area.x + area.width;
                    let previous_state = self.state;
                    self.state = if is_over_button {
                        match self.state {
                            State::Normal => State::Hover,
                            _ => self.state
                        }
                    } else {
                        match self.state {
                            State::Hover => State::Normal,
                            _ => self.state
                        }
                    };
                    previous_state != self.state
                } else {
                    false
                }
            },
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(area) = self.area {
                    let is_over_button = 
                        mouse_event.row >= area.y && 
                        mouse_event.row < area.y + area.height &&
                        mouse_event.column >= area.x && 
                        mouse_event.column < area.x + area.width;

                    if is_over_button {
                        self.state = State::Active;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            MouseEventKind::Up(MouseButton::Left) => {
                if self.state == State::Active {
                    self.state = State::Normal;
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }
}

impl Widget for Button<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        self.area = Some(area);

        let (background, text, shadow, highlight) = self.colors();
        buf.set_style(area, Style::new().bg(background).fg(text));

        if area.height > 2 {
            buf.set_string(
                area.x,
                area.y,
                "▔".repeat(area.width as usize),
                Style::new().fg(highlight).bg(background),
            );
        }
        if area.height > 1 {
            buf.set_string(
                area.x,
                area.y + area.height - 1,
                "▁".repeat(area.width as usize),
                Style::new().fg(shadow).bg(background),
            );
        }
        buf.set_line(
            area.x + (area.width.saturating_sub(self.label.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &self.label,
            area.width,
        );
    }
}

impl Button<'_> {
    const fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.state {
            State::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            State::Hover => (theme.hover, theme.text, theme.shadow, theme.highlight),
            State::Selected => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            State::Active => (theme.background, theme.text, theme.highlight, theme.shadow),
        }
    }
}