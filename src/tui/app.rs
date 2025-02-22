use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use tui_textarea::TextArea;
use tui_tree_widget::{TreeItem, TreeState};

use super::navigation::{NavigationManager, PaneId};
use super::widgets::searchable_textarea::SearchableTextArea;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
    Search,
}

pub struct SearchBox<'a> {
    pub textarea: TextArea<'a>,
    pub open: bool,
    pub replace_mode: bool,
}

impl Default for SearchBox<'_> {
    fn default() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
        );
        Self {
            textarea,
            open: false,
            replace_mode: false,
        }
    }
}

pub struct App<'a> {
    pub command_input: String,
    pub mode: Mode,
    pub message: String,

    pub selected_collection: Option<String>,

    pub navigation: NavigationManager,

    pub collection_state: TreeState<String>,
    pub collection_items: Vec<TreeItem<'a, String>>,
    pub workspace: SearchableTextArea<'a>,
    pub search: SearchBox<'a>,

    pub should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let mut workspace = SearchableTextArea::default();
        workspace.init();

        let collection_items = super::panes::collections::CollectionsPane::load_collections();
        
        let mut navigation = NavigationManager::new();
        navigation.register_pane(PaneId::Header, 1);
        navigation.register_pane(PaneId::Collections, 1);
        navigation.register_pane(PaneId::Workspace, 1);
        navigation.register_pane(PaneId::Results, 1);

        let collection = Some("PostgreSQL".to_string());
        
        Self {
            command_input: String::new(),
            mode: Mode::Normal,
            message: String::new(),
            selected_collection: collection,
            navigation,
            collection_state: TreeState::default(),
            collection_items,
            workspace,
            search: SearchBox::default(),
            should_quit: false,
        }
    }

    pub fn tick(&mut self) {
        // Update any app state that needs to change every tick
    }

    pub fn handle_key(&mut self, ui: &super::ui::UI, key_event: KeyEvent) -> anyhow::Result<bool> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(true);
            }
            (KeyCode::Tab, _) if !self.is_edit_mode() => {
                _ = self.navigation.cycle_pane(false);
                return Ok(false);
            }
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                self.mode = Mode::Command;
                return Ok(false);
            }
            _ => {
                return ui.handle_key_event(self, key_event);
            }
        }
    }

    pub fn handle_mouse(&mut self, ui: &super::ui::UI, mouse_event: MouseEvent) -> anyhow::Result<bool> {
        ui.handle_mouse_event(self, mouse_event)
    }

    // fn handle_search_mode(&mut self, key_event: KeyEvent) -> anyhow::Result<bool> {
    //     let input = Input::from(key_event);
    //     match input {
    //         Input { key: Key::Esc, .. } => {
    //             self.search.open = false;
    //             self.mode = Mode::Normal;
    //             self.workspace.set_search_pattern("")?;
    //             Ok(false)
    //         }
    //         Input { key: Key::Enter, .. } => {
    //             if self.search.replace_mode {
    //                 let pattern = self.search.textarea.lines()[0].as_str();
    //                 let replacement = self.search.textarea.lines().get(1).map(|s| s.as_str()).unwrap_or("");
    //                 self.workspace.set_search_pattern(pattern)?;
    //                 if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
    //                     let count = self.workspace.replace_all(replacement);
    //                     self.message = format!("Replaced {} occurrences", count);
    //                 } else {
    //                     if self.workspace.replace_next(replacement) {
    //                         self.message = "Replaced occurrence".to_string();
    //                     } else {
    //                         self.message = "No more matches".to_string();
    //                     }
    //                 }
    //             } else {
    //                 let pattern = self.search.textarea.lines()[0].as_str();
    //                 self.workspace.set_search_pattern(pattern)?;
    //                 if !self.workspace.search_forward(true) {
    //                     self.message = "Pattern not found".to_string();
    //                 }
    //             }
    //             self.search.open = false;
    //             self.mode = Mode::Normal;
    //             Ok(false)
    //         }
    //         Input { 
    //             key: Key::Char('n'),
    //             ctrl: true,
    //             ..
    //         } => {
    //             if !self.workspace.search_forward(false) {
    //                 self.message = "Pattern not found".to_string();
    //             }
    //             Ok(false)
    //         }
    //         Input {
    //             key: Key::Char('p'),
    //             ctrl: true,
    //             ..
    //         } => {
    //             if !self.workspace.search_back(false) {
    //                 self.message = "Pattern not found".to_string();
    //             }
    //             Ok(false)
    //         }
    //         _ => {
    //             self.search.textarea.input(input);
    //             if let Some(pattern) = self.search.textarea.lines().first() {
    //                 self.workspace.set_search_pattern(pattern)?;
    //             }
    //             Ok(false)
    //         }
    //     }
    // }

    pub fn execute_query(&mut self) {
        // TODO
    }

    pub fn save_query(&mut self) {
        let content = self.workspace.get_content();
        if !content.is_empty() {
            // TODO
        }
    }
    
    pub fn is_collections_active(&self) -> bool {
        self.navigation.is_active(PaneId::Collections)
    }
    
    pub fn is_workspace_active(&self) -> bool {
        self.navigation.is_active(PaneId::Workspace)
    }
    
    pub fn is_results_active(&self) -> bool {
        self.navigation.is_active(PaneId::Results)
    }

    pub fn is_edit_mode(&self) -> bool {
        [PaneId::Collections, PaneId::Workspace, PaneId::Results]
            .iter()
            .any(|&pane_id| self.is_pane_in_edit_mode(pane_id))
    }
    
    pub fn is_pane_in_edit_mode(&self, pane_id: PaneId) -> bool {
        if let Some(info) = self.navigation.get_pane_info(pane_id) {
            info.is_editing()
        } else {
            false
        }
    }
}