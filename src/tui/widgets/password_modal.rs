use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Rect, style::{Color, Style}, widgets::{Block, Borders}, Frame};
use std::any::Any;
use tui_textarea::TextArea;

use crate::tui::widgets::button::{RED, GREEN};
use super::modal::{DialogButton, DialogContent, ModalAction, ModalDialog, ModalHandler};

pub struct PasswordModal {
    textarea: TextArea<'static>,
}

impl Default for PasswordModal {
    fn default() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_style(Style::default().bg(Color::Black));
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightBlue))
                .title("Password")
        );
        textarea.set_mask_char('*');

        Self {
            textarea,
        }
    }
}

impl ModalHandler for PasswordModal {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModalAction> {
        use crossterm::event::KeyCode;
        
        match key_event.code {
            KeyCode::Enter => Ok(ModalAction::Custom("submit".to_string())),
            KeyCode::Esc => Ok(ModalAction::Close),
            _ => {
                self.textarea.input(tui_textarea::Input::from(key_event));
                Ok(ModalAction::None)
            }
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<ModalAction> {
        let content = DialogContent {
            title: "Enter Password",
            content_widget: &self.textarea,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(RED),
                DialogButton::new("Submit", "submit").with_theme(GREEN),
            ],
        };

        let dialog = ModalDialog::new(content);
        dialog.handle_mouse_event(mouse_event, area)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content = DialogContent {
            title: "Enter Password",
            content_widget: &self.textarea,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(RED),
                DialogButton::new("Submit", "submit").with_theme(GREEN),
            ],
        };

        let dialog = ModalDialog::new(content)
            .with_dimensions(50, 30);
        frame.render_widget(dialog, area);
    }
}

impl PasswordModal {
    pub fn get_password(&self) -> Option<String> {
        self.textarea.lines().first().map(|s| s.to_string())
    }
}