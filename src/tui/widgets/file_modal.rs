use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
    Frame,
};
use std::any::Any;

use crate::{
    collection::CollectionScope,
    tui::widgets::{
        button::{RED, GREEN},
        modal::{DialogButton, DialogContent, ModalAction, ModalDialog, ModalHandler},
        radio_group::{RadioGroup, RadioOption},
    },
};

struct NewFileContent<'a> {
    name_input: &'a tui_textarea::TextArea<'a>,
    type_selector: &'a RadioGroup<'a>,
    scope_selector: &'a RadioGroup<'a>,
    focused_element: usize,
}

impl<'a> Widget for NewFileContent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Name input
                Constraint::Length(3), // Type selector
                Constraint::Length(3), // Scope selector
            ])
            .split(area);

        Widget::render(self.name_input, chunks[0], buf);
        
        let type_selector = self.type_selector.clone();
        let scope_selector = self.scope_selector.clone();
        
        Widget::render(type_selector, chunks[1], buf);
        Widget::render(scope_selector, chunks[2], buf);

        let focus_style = Style::default().fg(Color::Yellow);
        match self.focused_element {
            0 => buf.set_style(chunks[0], focus_style),
            1 => buf.set_style(chunks[1], focus_style),
            2 => buf.set_style(chunks[2], focus_style),
            _ => {}
        }
    }
}

struct EditFileContent<'a> {
    name_input: &'a tui_textarea::TextArea<'a>,
    scope_selector: &'a RadioGroup<'a>,
    is_folder: bool,
    focused_element: usize,
}

impl<'a> Widget for EditFileContent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut constraints = vec![Constraint::Length(3)]; // Name input
        if self.is_folder {
            constraints.push(Constraint::Length(3)); // Scope selector for folders
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints)
            .split(area);

        Widget::render(self.name_input, chunks[0], buf);
        
        if self.is_folder {
            let scope_selector = self.scope_selector.clone();
            Widget::render(scope_selector, chunks[1], buf);
        }

        let focus_style = Style::default().fg(Color::Yellow);
        match self.focused_element {
            0 => buf.set_style(chunks[0], focus_style),
            1 if self.is_folder => buf.set_style(chunks[1], focus_style),
            _ => {}
        }
    }
}

pub struct NewFileModal {
    name_input: tui_textarea::TextArea<'static>,
    type_selector: RadioGroup<'static>,
    scope_selector: RadioGroup<'static>,
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
            RadioOption { label: "User", value: "user".to_string() },
            RadioOption { label: "Local", value: "local".to_string() },
        ];
        let scope_selector = RadioGroup::new(scope_options)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::LightBlue));

        Self {
            name_input,
            type_selector,
            scope_selector,
            focused_element: 0,
        }
    }
}

impl ModalHandler for NewFileModal {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModalAction> {
        match key_event.code {
            KeyCode::Enter => Ok(ModalAction::Custom("submit".to_string())),
            KeyCode::Esc => Ok(ModalAction::Close),
            KeyCode::Tab => {
                self.focused_element = (self.focused_element + 1) % 3;
                Ok(ModalAction::None)
            }
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

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<ModalAction> {
        let content_widget = NewFileContent {
            name_input: &self.name_input,
            type_selector: &self.type_selector,
            scope_selector: &self.scope_selector,
            focused_element: self.focused_element,
        };

        let content = DialogContent {
            title: "New File/Folder",
            content_widget,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(RED),
                DialogButton::new("Create", "submit").with_theme(GREEN),
            ],
        };

        ModalDialog::new(content)
            .with_dimensions(50, 40)
            .handle_mouse_event(mouse_event, area)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content_widget = NewFileContent {
            name_input: &self.name_input,
            type_selector: &self.type_selector,
            scope_selector: &self.scope_selector,
            focused_element: self.focused_element,
        };

        let content = DialogContent {
            title: "New File/Folder",
            content_widget,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(RED),
                DialogButton::new("Create", "submit").with_theme(GREEN),
            ],
        };

        let dialog = ModalDialog::new(content)
            .with_dimensions(50, 40);
            
        frame.render_widget(dialog, area);
    }
}

impl NewFileModal {
    pub fn get_values(&self) -> (String, String, CollectionScope) {
        let name = self.name_input.lines().first()
            .map(|s| s.to_string())
            .unwrap_or_default();
            
        let file_type = self.type_selector.selected_value();
        let scope = match self.scope_selector.selected_value().as_str() {
            "user" => CollectionScope::User,
            _ => CollectionScope::Cwd,
        };

        (name, file_type, scope)
    }
}

pub struct EditFileModal {
    name_input: tui_textarea::TextArea<'static>,
    scope_selector: RadioGroup<'static>,
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
            RadioOption { label: "User", value: "user".to_string() },
            RadioOption { label: "Local", value: "local".to_string() },
        ];
        let mut scope_selector = RadioGroup::new(scope_options)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::LightBlue));
            
        scope_selector.set_selected(match current_scope {
            CollectionScope::User => 0,
            CollectionScope::Cwd => 1,
        });

        Self {
            name_input,
            scope_selector,
            is_folder,
            focused_element: 0,
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

impl ModalHandler for EditFileModal {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModalAction> {
        match key_event.code {
            KeyCode::Enter => Ok(ModalAction::Custom("submit".to_string())),
            KeyCode::Esc => Ok(ModalAction::Close),
            KeyCode::Tab => {
                if self.is_folder {
                    self.focused_element = (self.focused_element + 1) % 2;
                }
                Ok(ModalAction::None)
            }
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

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent, area: Rect) -> Result<ModalAction> {
        let content_widget = EditFileContent {
            name_input: &self.name_input,
            scope_selector: &self.scope_selector,
            is_folder: self.is_folder,
            focused_element: self.focused_element,
        };

        let content = DialogContent {
            title: if self.is_folder { "Edit Folder" } else { "Edit File" },
            content_widget,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(RED),
                DialogButton::new("Save", "submit").with_theme(GREEN),
            ],
        };

        ModalDialog::new(content)
            .with_dimensions(50, if self.is_folder { 35 } else { 25 })
            .handle_mouse_event(mouse_event, area)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content_widget = EditFileContent {
            name_input: &self.name_input,
            scope_selector: &self.scope_selector,
            is_folder: self.is_folder,
            focused_element: self.focused_element,
        };

        let content = DialogContent {
            title: if self.is_folder { "Edit Folder" } else { "Edit File" },
            content_widget,
            buttons: vec![
                DialogButton::new("Cancel", "cancel").with_theme(RED),
                DialogButton::new("Save", "submit").with_theme(GREEN),
            ],
        };

        let dialog = ModalDialog::new(content)
            .with_dimensions(50, if self.is_folder { 35 } else { 25 });
            
        frame.render_widget(dialog, area);
    }
}