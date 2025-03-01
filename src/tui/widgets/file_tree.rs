use anyhow::Result;
use crossterm::event::{MouseEvent, MouseButton, MouseEventKind};
use ratatui::{
    layout::{Position, Rect},
    widgets::StatefulWidget,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

/// A wrapper around `tui_tree_widget::Tree` that adds click-to-select functionality
pub struct FileTree<'a, Identifier> {
    inner: Tree<'a, Identifier>,
}

impl<'a, Identifier> FileTree<'a, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    pub fn new(items: &'a [TreeItem<'a, Identifier>]) -> std::io::Result<Self> {
        Ok(Self {
            inner: Tree::new(items)?,
        })
    }

    /// Delegate all method calls to the inner Tree widget
    pub fn block(mut self, block: ratatui::widgets::Block<'a>) -> Self {
        self.inner = self.inner.block(block);
        self
    }

    pub fn style(mut self, style: ratatui::style::Style) -> Self {
        self.inner = self.inner.style(style);
        self
    }

    pub fn highlight_style(mut self, style: ratatui::style::Style) -> Self {
        self.inner = self.inner.highlight_style(style);
        self
    }

    pub fn highlight_symbol(mut self, symbol: &'a str) -> Self {
        self.inner = self.inner.highlight_symbol(symbol);
        self
    }

    pub fn experimental_scrollbar(mut self, scrollbar: Option<ratatui::widgets::Scrollbar<'a>>) -> Self {
        self.inner = self.inner.experimental_scrollbar(scrollbar);
        self
    }

    pub fn handle_mouse_event(
        &self,
        state: &mut TreeState<Identifier>,
        mouse_event: MouseEvent,
        area: Rect,
    ) -> Result<bool> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let position = Position::new(mouse_event.column, mouse_event.row);
                
                if !area.contains(position) {
                    return Ok(false);
                }
                
                if let Some(identifier) = state.rendered_at(position) {
                    let was_selected = identifier == state.selected();
                    state.select(identifier.to_vec());
                    
                    if was_selected {
                        state.toggle_selected();
                    }
                    
                    return Ok(true);
                }
            }
            _ => {}
        }
        Ok(false)
    }
}

impl<'a, Identifier> StatefulWidget for FileTree<'a, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    type State = TreeState<Identifier>;

    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer, state: &mut Self::State) {
        self.inner.render(area, buf, state);
    }
}