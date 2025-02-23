use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame, layout::Rect,
};
use tui_textarea::TextArea;

use super::{button::{GREEN, RED}, modal::{DialogButton, DialogContent, ModalDialog}};

pub struct PasswordModal<'a> {
    pub textarea: TextArea<'a>,
    pub error: Option<String>,
}

impl<'a> Default for PasswordModal<'a> {
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
            error: None,
        }
    }
}

impl<'a> PasswordModal<'a> {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let content = DialogContent {
            title: "Enter Password",
            content_widget: &self.textarea,
            buttons: vec![
                DialogButton::new(
                    "Cancel",
                    Box::new(|| {
                        // Handle cancel
                        Ok(())
                    })
                ).with_theme(RED),
                DialogButton::new(
                    "Submit",
                    Box::new(|| {
                        // Handle submit
                        Ok(())
                    })
                ).with_theme(GREEN),
            ],
        };

        let dialog = ModalDialog::new(content, None, Some(25));
        frame.render_widget(dialog, area);
    }
}