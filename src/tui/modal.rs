use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{prelude::*, Frame};
use std::any::Any;

use crate::collection::CollectionScope;

use super::widgets::{file_modal::{EditFileModal, NewFileModal}, modal::{ModalAction, ModalHandler}, password_modal::PasswordModal};

pub enum ModalType {
    Password,
    NewFile,
    EditFile {
        name: String,
        is_folder: bool,
        current_scope: CollectionScope,
    },
}

pub struct ModalManager {
    active_modal: Option<Box<dyn ModalHandler>>,
    modal_result: Option<String>,
}

impl ModalManager {
    pub fn new() -> Self {
        Self {
            active_modal: None,
            modal_result: None,
        }
    }

    pub fn show_modal(&mut self, modal_type: ModalType) {
        let modal: Box<dyn ModalHandler> = match modal_type {
            ModalType::Password => Box::new(PasswordModal::default()),
            ModalType::NewFile => Box::new(NewFileModal::default()),
            ModalType::EditFile { name, is_folder, current_scope } => {
                Box::new(EditFileModal::new(&name, is_folder, current_scope))
            }
        };
        self.active_modal = Some(modal);
    }

    pub fn close_modal(&mut self) {
        self.active_modal = None;
        self.modal_result = None;
    }

    pub fn is_modal_active(&self) -> bool {
        self.active_modal.is_some()
    }

    pub fn get_modal_mut(&mut self) -> Option<&mut Box<dyn ModalHandler>> {
        self.active_modal.as_mut()
    }

    pub fn get_active_modal_as<T: Any>(&mut self) -> Option<&mut T> {
        self.active_modal
            .as_mut()
            .and_then(|modal| modal.as_any_mut().downcast_mut::<T>())
    }

    pub fn take_result(&mut self) -> Option<String> {
        self.modal_result.take()
    }

    pub fn handle_event(&mut self, event: ModalEvent) -> Result<ModalAction> {
        if let Some(modal) = &mut self.active_modal {
            match event {
                ModalEvent::Key(key_event) => modal.handle_key_event(key_event),
                ModalEvent::Mouse(mouse_event, area) => modal.handle_mouse_event(mouse_event, area),
            }
        } else {
            Ok(ModalAction::None)
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(modal) = &mut self.active_modal {
            modal.render(frame, area);
        }
    }

    pub fn store_result(&mut self, result: String) {
        self.modal_result = Some(result);
    }
}

pub enum ModalEvent {
    Key(KeyEvent),
    Mouse(MouseEvent, Rect),
}