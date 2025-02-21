use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::text::Line;
use anyhow::Result;

use crate::tui::app::App;

pub trait Instructions {
    fn get_instructions(&self, app: &App) -> Line;
}

pub trait PaneEventHandler {
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> Result<bool>;
    fn handle_mouse_event(&self, app: &mut App, mouse_event: MouseEvent) -> Result<bool>;
}