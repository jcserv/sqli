use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};

/// A custom button widget inspired by ratatui's example
#[derive(Debug, Clone)]
pub struct Button<'a> {
    /// Text to display in the button
    label: Line<'a>,
    /// Style for the button when not focused
    normal_style: Style,
    /// Style for the button when focused
    focused_style: Style,
    /// Style for the button when pressed
    pressed_style: Style,
    /// Current state of the button
    state: ButtonState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Focused,
    Pressed,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            label: Line::from(text),
            normal_style: Style::default(),
            focused_style: Style::default().fg(Color::LightBlue),
            pressed_style: Style::default().fg(Color::Black).bg(Color::LightBlue),
            state: ButtonState::Normal,
        }
    }

    pub const fn normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    pub const fn focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    pub const fn pressed_style(mut self, style: Style) -> Self {
        self.pressed_style = style;
        self
    }

    pub const fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent, has_focus: bool) -> bool {
        if !has_focus {
            return false;
        }

        match key_event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.state = ButtonState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> bool {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let position = Position::new(mouse_event.column, mouse_event.row);
                if area.contains(position) {
                    self.state = ButtonState::Pressed;
                    return true;
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if self.state == ButtonState::Pressed {
                    self.state = ButtonState::Focused;
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    pub fn is_pressed(&self) -> bool {
        self.state == ButtonState::Pressed
    }

    pub fn reset(&mut self) {
        self.state = ButtonState::Normal;
    }

    pub fn set_focused(&mut self) {
        if self.state != ButtonState::Pressed {
            self.state = ButtonState::Focused;
        }
    }

    pub fn set_normal(&mut self) {
        if self.state != ButtonState::Pressed {
            self.state = ButtonState::Normal;
        }
    }
}

impl<'a> Widget for Button<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = match self.state {
            ButtonState::Normal => self.normal_style,
            ButtonState::Focused => self.focused_style,
            ButtonState::Pressed => self.pressed_style,
        };

        buf.set_style(area, style);

        let x_offset = (area.width.saturating_sub(self.label.width() as u16)) / 2;
        let y_offset = (area.height.saturating_sub(1)) / 2;

        let (x_shift, y_shift) = match self.state {
            ButtonState::Pressed => (1, 0),
            _ => (0, 0),
        };

        buf.set_line(
            area.x + x_offset + x_shift.min(1),
            area.y + y_offset + y_shift.min(1),
            &self.label,
            area.width.saturating_sub(x_shift),
        );
    }
}