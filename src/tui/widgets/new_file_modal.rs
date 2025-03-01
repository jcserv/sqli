use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
    Frame,
};
use std::{any::Any, cmp::min};

use crate::{
    collection::CollectionScope,
    tui::widgets::{
        button::{LIGHT_GREY, GREEN},
        modal::{DialogButton, DialogContent, ModalAction, ModalDialog, ModalHandler},
        radio_group::{RadioGroup, RadioOption},
    },
};

use super::modal::FocusableArea;

struct NewFileContent<'a> {
    name_input: &'a tui_textarea::TextArea<'a>,
    type_selector: &'a RadioGroup<'a>,
    scope_selector: &'a RadioGroup<'a>,
    focused_element: usize,
}

impl Default for NewFileModal {
    fn default() -> Self {
        let mut name_input = tui_textarea::TextArea::default();
        name_input.set_style(Style::default().bg(Color::Black));
        name_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightBlue))
                .title("Name")
        );

        let type_options = vec![
            RadioOption { label: "File", value: "file".to_string() },
            RadioOption { label: "Folder", value: "folder".to_string() },
        ];
        let type_selector = RadioGroup::new(type_options)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::LightBlue));

        let scope_options = vec![
            RadioOption { label: "Local", value: "local".to_string() },
            RadioOption { label: "User", value: "user".to_string() },
        ];
        let scope_selector = RadioGroup::new(scope_options)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::LightBlue));

        Self {
            name_input,
            type_selector,
            scope_selector,
            focused_element: 0,
            parent_folder: None,
        }
    }
}


impl Widget for NewFileContent<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Name input
                Constraint::Length(1),  // Gap
                Constraint::Length(1),  // Type selector
                Constraint::Length(1),  // Scope selector
            ])
            .split(area);

        let type_selector = self.type_selector.clone();
        let scope_selector = self.scope_selector.clone();

        Widget::render(self.name_input, chunks[0], buf);
        Widget::render(type_selector, chunks[2], buf);
        Widget::render(scope_selector, chunks[3], buf);

        let focus_style = Style::default().fg(Color::Yellow);
        match self.focused_element {
            0 => buf.set_style(chunks[0], focus_style),
            1 => buf.set_style(chunks[2], focus_style),
            2 => buf.set_style(chunks[3], focus_style),
            _ => {}
        }
    }
}

pub struct NewFileModal {
    name_input: tui_textarea::TextArea<'static>,
    type_selector: RadioGroup<'static>,
    scope_selector: RadioGroup<'static>,
    focused_element: usize,
    parent_folder: Option<String>,
}

impl ModalHandler for NewFileModal {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModalAction> {
        match key_event.code {
            KeyCode::Tab => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.handle_tab(true)
                } else {
                    self.handle_tab(false)
                }
            },
            KeyCode::BackTab => {
                self.handle_tab(true)
            },
            KeyCode::Enter => {
                match self.focused_element {
                    3 => Ok(ModalAction::Custom("cancel".to_string())), // Cancel button
                    4 => Ok(ModalAction::Custom("submit".to_string())), // Create button
                    _ => Ok(ModalAction::None),
                }
            },
            KeyCode::Esc => Ok(ModalAction::Close),
            _ => {
                match self.focused_element {
                    0 => {
                        self.name_input.input(tui_textarea::Input::from(key_event));
                    }
                    1 => {
                        self.type_selector.handle_key_event(key_event);
                    }
                    2 => {
                        self.scope_selector.handle_key_event(key_event);
                    }
                    _ => {}
                }
                Ok(ModalAction::None)
            }
        }
    }

    fn handle_tab(&mut self, reverse: bool) -> Result<ModalAction> {
        if reverse {
            self.focused_element = if self.focused_element == 0 { 4 } else { self.focused_element - 1 };
        } else {
            self.focused_element = (self.focused_element + 1) % 5;
        }
        Ok(ModalAction::None)
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<ModalAction> {
        let content_widget = NewFileContent {
            name_input: &self.name_input,
            type_selector: &self.type_selector,
            scope_selector: &self.scope_selector,
            focused_element: min(self.focused_element, 2),
        };

        let content = DialogContent {
            title: "New File/Folder",
            content_widget,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(LIGHT_GREY),
                DialogButton::new("Create", "submit").with_theme(GREEN),
            ],
        };

        let mut dialog = ModalDialog::new(content)
            .with_dimensions(50, 40)
            .with_content_element_count(3);
            
        let focused_area = if self.focused_element <= 2 {
            FocusableArea::Content(self.focused_element)
        } else {
            FocusableArea::Button(self.focused_element - 3)
        };
        
        dialog = dialog.with_focused_area(focused_area);
        
        let result = dialog.handle_mouse_event(mouse_event, area)?;
        if let ModalAction::Custom(action) = &result {
            if action == "cancel" {
                self.focused_element = 3;
            } else if action == "submit" {
                self.focused_element = 4;
            }
        }
        
        Ok(result)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content_widget = NewFileContent {
            name_input: &self.name_input,
            type_selector: &self.type_selector,
            scope_selector: &self.scope_selector,
            focused_element: min(self.focused_element, 2),
        };

        let content = DialogContent {
            title: "New File/Folder",
            content_widget,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(LIGHT_GREY),
                DialogButton::new("Create", "submit").with_theme(GREEN),
            ],
        };

        let mut dialog = ModalDialog::new(content)
            .with_dimensions(50, 40)
            .with_content_element_count(3);
            
        let focused_area = if self.focused_element <= 2 {
            FocusableArea::Content(self.focused_element)
        } else {
            FocusableArea::Button(self.focused_element - 3)
        };
        
        dialog = dialog.with_focused_area(focused_area);    
        frame.render_widget(dialog, area);
    }
}

impl NewFileModal {
    pub fn with_parent_folder(mut self, parent_folder: String) -> Self {
        self.parent_folder = Some(parent_folder);
        self
    }

    pub fn get_values(&self) -> (String, String, CollectionScope, Option<String>) {
        let name = self.name_input.lines().first()
            .map(|s| s.to_string())
            .unwrap_or_default();
            
        let file_type = self.type_selector.selected_value();
        let scope = match self.scope_selector.selected_value().as_str() {
            "user" => CollectionScope::User,
            _ => CollectionScope::Cwd,
        };

        (name, file_type, scope, self.parent_folder.clone())
    }
}