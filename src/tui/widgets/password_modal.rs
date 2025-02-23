use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, widgets::{Block, Borders}, Frame
};
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use tui_textarea::TextArea;

use super::{button::{GREEN, RED}, modal::{DialogButton, DialogContent, ModalDialog}};

pub struct PasswordModal<'a> {
    pub textarea: TextArea<'a>,
    pub error: Option<String>,
    button_areas: Option<Vec<Rect>>,
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
            button_areas: None,
        }
    }
}

impl<'a> PasswordModal<'a> {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let content = DialogContent {
            title: "Enter Password",
            content_widget: &self.textarea,
            buttons: vec![
                DialogButton::new(
                    "Cancel",
                    Box::new(|| Ok(()))
                ).with_theme(RED),
                DialogButton::new(
                    "Submit",
                    Box::new(|| Ok(()))
                ).with_theme(GREEN),
            ],
        };

        let dialog = ModalDialog::new(content, None, Some(25));
        
        let modal_area = centered_rect(40, 25, area);
        let inner_area = modal_area.inner(Default::default());
        let button_areas = calculate_button_areas(inner_area);
        self.button_areas = Some(button_areas);
        
        frame.render_widget(dialog, area);
    }

    pub fn handle_mouse_event(&self, event: MouseEvent) -> Option<PasswordAction> {
        if let MouseEventKind::Down(MouseButton::Left) = event.kind {
            if let Some(areas) = &self.button_areas {
                let mouse_pos = (event.column, event.row);
                
                if point_in_rect(mouse_pos, areas[0]) {
                    return Some(PasswordAction::Cancel);
                }
                
                if point_in_rect(mouse_pos, areas[1]) {
                    return Some(PasswordAction::Submit);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PasswordAction {
    Cancel,
    Submit,
}

fn point_in_rect(point: (u16, u16), rect: Rect) -> bool {
    point.0 >= rect.x 
        && point.0 < rect.x + rect.width
        && point.1 >= rect.y 
        && point.1 < rect.y + rect.height
}

fn calculate_button_areas(area: Rect) -> Vec<Rect> {
    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(12), // Cancel button
            Constraint::Length(2),  // Gap
            Constraint::Length(12), // Submit button
            Constraint::Percentage(30),
        ])
        .split(Rect::new(
            area.x,
            area.y + area.height - 3,
            area.width,
            3
        ));

    vec![button_layout[1], button_layout[3]]
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}