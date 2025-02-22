use ratatui::text::Line;

use crate::tui::app::App;

pub trait Instructions {
    fn get_instructions(&self, app: &App) -> Line;
}