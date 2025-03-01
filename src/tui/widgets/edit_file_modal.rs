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

use super::{button::RED, modal::FocusableArea};

pub struct EditFileModal {
    name_input: tui_textarea::TextArea<'static>,
    scope_selector: RadioGroup<'static>,
    is_folder: bool,
    focused_element: usize,
    element_count: usize,
}

struct EditFileContent<'a> {
    name_input: &'a tui_textarea::TextArea<'a>,
    scope_selector: &'a RadioGroup<'a>,
    is_folder: bool,
    focused_element: usize,
}

impl EditFileModal {
    pub fn new(name: &str, is_folder: bool, current_scope: CollectionScope) -> Self {
        let mut name_input = tui_textarea::TextArea::default();
        name_input.set_style(Style::default().bg(Color::Black));
        name_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightBlue))
                .title("Name")
        );
        name_input.insert_str(name);

        let scope_options = vec![
            RadioOption { label: "Local", value: "local".to_string() },
            RadioOption { label: "User", value: "user".to_string() },
        ];
        let mut scope_selector = RadioGroup::new(scope_options)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::LightBlue));
            
        scope_selector.set_selected(match current_scope {
            CollectionScope::Cwd => 0,
            CollectionScope::User => 1,
        });

        Self {
            name_input,
            scope_selector,
            is_folder,
            focused_element: 0,
            element_count: if is_folder { 2 } else { 1 }, 
        }
    }
    
    pub fn get_values(&self) -> (String, CollectionScope) {
        let name = self.name_input.lines().first()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let scope = match self.scope_selector.selected_value().as_str() {
            "user" => CollectionScope::User,
            _ => CollectionScope::Cwd,
        };

        (name, scope)
    }
}

impl<'a> Widget for EditFileContent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut constraints = vec![
            Constraint::Length(3),  // Name input
        ];
        
        if self.is_folder {
            constraints.extend_from_slice(&[
                Constraint::Length(1),  // Gap
                Constraint::Length(1),  // Scope selector
            ]);
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        Widget::render(self.name_input, chunks[0], buf);
        
        if self.is_folder {
            let scope_selector = self.scope_selector.clone();
            Widget::render(scope_selector, chunks[2], buf);
        }

        let focus_style = Style::default().fg(Color::Yellow);
        match self.focused_element {
            0 => buf.set_style(chunks[0], focus_style),
            1 if self.is_folder => buf.set_style(chunks[2], focus_style),
            _ => {}
        }
    }
}

impl ModalHandler for EditFileModal {
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
                match self.focused_element {
                    idx if idx == self.element_count => Ok(ModalAction::Custom("cancel".to_string())),
                    idx if idx == self.element_count + 1 => Ok(ModalAction::Custom("delete".to_string())), 
                    idx if idx == self.element_count + 2 => Ok(ModalAction::Custom("edit".to_string())),
                    _ => Ok(ModalAction::None),
                }
            },
            KeyCode::Esc => Ok(ModalAction::Close),
            _ => {
                match self.focused_element {
                    0 => {
                        self.name_input.input(tui_textarea::Input::from(key_event));
                    }
                    1 if self.is_folder => {
                        self.scope_selector.handle_key_event(key_event);
                    }
                    _ => {}
                }
                Ok(ModalAction::None)
            }
        }
    }

    fn handle_tab(&mut self, reverse: bool) -> Result<ModalAction> {
        let total_elements = self.element_count + 3;
        
        if reverse {
            self.focused_element = if self.focused_element == 0 { 
                total_elements - 1 
            } else { 
                self.focused_element - 1 
            };
        } else {
            self.focused_element = (self.focused_element + 1) % total_elements;
        }
        
        Ok(ModalAction::None)
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<ModalAction> {
        let content_widget = EditFileContent {
            name_input: &self.name_input,
            scope_selector: &self.scope_selector,
            is_folder: self.is_folder,
            focused_element: min(self.focused_element, self.element_count - 1),
        };

        let content = DialogContent {
            title: if self.is_folder { "Edit Folder" } else { "Edit File" },
            content_widget,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(LIGHT_GREY),
                DialogButton::new("Delete", "delete").with_theme(RED),
                DialogButton::new("Save", "edit").with_theme(GREEN),
            ],
        };

        let mut dialog = ModalDialog::new(content)
            .with_dimensions(50, if self.is_folder { 35 } else { 25 })
            .with_content_element_count(self.element_count);
        
        let focused_area = if self.focused_element < self.element_count {
            FocusableArea::Content(self.focused_element)
        } else {
            FocusableArea::Button(self.focused_element - self.element_count)
        };
        
        dialog = dialog.with_focused_area(focused_area);
        
        let result = dialog.handle_mouse_event(mouse_event, area)?;        
        if let ModalAction::Custom(action) = &result {
            match action.as_str() {
                "cancel" => self.focused_element = self.element_count,
                "delete" => self.focused_element = self.element_count + 1,
                "edit" => self.focused_element = self.element_count + 2,
                _ => {}
            }
        }
        
        Ok(result)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content_widget = EditFileContent {
            name_input: &self.name_input,
            scope_selector: &self.scope_selector,
            is_folder: self.is_folder,
            focused_element: min(self.focused_element, self.element_count - 1),
        };

        let content = DialogContent {
            title: if self.is_folder { "Edit Folder" } else { "Edit File" },
            content_widget,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(LIGHT_GREY),
                DialogButton::new("Delete", "delete").with_theme(RED),
                DialogButton::new("Save", "edit").with_theme(GREEN),
            ],
        };

        let mut dialog = ModalDialog::new(content)
            .with_dimensions(50, if self.is_folder { 35 } else { 25 })
            .with_content_element_count(self.element_count);
        
        let focused_area = if self.focused_element < self.element_count {
            FocusableArea::Content(self.focused_element)
        } else {
            FocusableArea::Button(self.focused_element - self.element_count)
        };
        
        dialog = dialog.with_focused_area(focused_area);
        
        frame.render_widget(dialog, area);
    }
}