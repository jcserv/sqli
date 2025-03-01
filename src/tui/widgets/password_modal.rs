use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{layout::Rect, style::{Color, Style}, widgets::{Block, Borders}, Frame};
use std::any::Any;
use tui_textarea::TextArea;

use crate::tui::widgets::button::{LIGHT_GREY, GREEN};
use super::modal::{DialogButton, DialogContent, FocusableArea, ModalAction, ModalDialog, ModalHandler};

pub struct PasswordModal {
    textarea: TextArea<'static>,
    focus_idx: usize,
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
            focus_idx: 0,
        }
    }
}

impl ModalHandler for PasswordModal {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModalAction> {
        match key_event.code {
            KeyCode::Tab => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    return self.handle_tab(true);
                } else {
                    return self.handle_tab(false);
                }
            },
            KeyCode::BackTab => {
                return self.handle_tab(true);
            },
            KeyCode::Enter => {
                match self.focus_idx {
                    0 => Ok(ModalAction::Custom("submit".to_string())),
                    1 => Ok(ModalAction::Custom("cancel".to_string())),
                    2 => Ok(ModalAction::Custom("submit".to_string())),
                    _ => Ok(ModalAction::None),
                }
            },
            KeyCode::Esc => Ok(ModalAction::Close),
            _ => {
                if self.focus_idx == 0 {
                    self.textarea.input(tui_textarea::Input::from(key_event));
                }
                Ok(ModalAction::None)
            }
        }
    }

    fn handle_tab(&mut self, reverse: bool) -> Result<ModalAction> {
        if reverse {
            self.focus_idx = if self.focus_idx == 0 { 2 } else { self.focus_idx - 1 };
        } else {
            self.focus_idx = (self.focus_idx + 1) % 3;
        }
        Ok(ModalAction::None)
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<ModalAction> {
        let content = DialogContent {
            title: "Enter Password",
            content_widget: &self.textarea,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(LIGHT_GREY),
                DialogButton::new("Submit", "submit").with_theme(GREEN),
            ],
        };

        let mut dialog = ModalDialog::new(content)
            .with_content_element_count(1);
        
        let focused_area = match self.focus_idx {
            0 => FocusableArea::Content(0),
            1 => FocusableArea::Button(0), // Cancel button
            2 => FocusableArea::Button(1), // Submit button
            _ => FocusableArea::Content(0),
        };
        dialog = dialog.with_focused_area(focused_area);
        
        let result = dialog.handle_mouse_event(mouse_event, area)?;        
        if let ModalAction::Custom(ref action) = result {
            if action == "cancel" {
                self.focus_idx = 1;
            } else if action == "submit" {
                self.focus_idx = 2;
            }
        }
        
        Ok(result)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content = DialogContent {
            title: "Enter Password",
            content_widget: &self.textarea,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(LIGHT_GREY),
                DialogButton::new("Submit", "submit").with_theme(GREEN),
            ],
        };

        let dialog = ModalDialog::new(content)
            .with_dimensions(50, 30)
            .with_content_element_count(1);
        
        let focused_area = match self.focus_idx {
            0 => FocusableArea::Content(0),
            1 => FocusableArea::Button(0), // Cancel button
            2 => FocusableArea::Button(1), // Submit button
            _ => FocusableArea::Content(0),
        };
        let dialog = dialog.with_focused_area(focused_area);
        
        frame.render_widget(dialog, area);
    }
}

impl PasswordModal {
    pub fn get_password(&self) -> Option<String> {
        self.textarea.lines().first().map(|s| s.to_string())
    }
}