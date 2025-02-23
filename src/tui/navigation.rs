use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneId {
    Header,
    Collections,
    Workspace,
    Results,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusType {
    /// The pane is not focused at all
    Inactive,
    /// The pane is focused but not in edit mode
    Active,
    /// The pane is focused and in edit mode
    Editing,
}

pub struct PaneInfo {
    pub id: PaneId,
    /// Number of tabbable elements within this pane
    pub element_count: usize,
    /// Current focused element within the pane (0-based)
    pub current_element: usize,
    /// Current focus state of this pane
    pub focus_type: FocusType,
}

impl PaneInfo {
    pub fn new(id: PaneId, element_count: usize) -> Self {
        Self {
            id,
            element_count,
            current_element: 0,
            focus_type: FocusType::Inactive,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.focus_type, FocusType::Active | FocusType::Editing)
    }
    
    pub fn is_editing(&self) -> bool {
        matches!(self.focus_type, FocusType::Editing)
    }
    
    pub fn activate(&mut self) {
        self.focus_type = FocusType::Active;
    }
    
    pub fn start_editing(&mut self) {
        self.focus_type = FocusType::Editing;
    }
    
    pub fn deactivate(&mut self) {
        self.focus_type = FocusType::Inactive;
    }
    
    pub fn stop_editing(&mut self) {
        self.focus_type = if self.focus_type == FocusType::Editing {
            FocusType::Active
        } else {
            self.focus_type
        };
    }
    
    pub fn next_element(&mut self) -> bool {
        if self.element_count <= 1 {
            return false;
        }
        
        self.current_element = (self.current_element + 1) % self.element_count;
        true
    }
    
    pub fn prev_element(&mut self) -> bool {
        if self.element_count <= 1 {
            return false;
        }
        
        self.current_element = if self.current_element == 0 {
            self.element_count - 1
        } else {
            self.current_element - 1
        };
        true
    }
}

pub struct NavigationManager {
    panes: HashMap<PaneId, PaneInfo>,
    tab_order: Vec<PaneId>,
    active_pane: Option<PaneId>,
}

impl NavigationManager {
    pub fn new() -> Self {
        Self {
            panes: HashMap::new(),
            tab_order: Vec::new(),
            active_pane: None,
        }
    }
    
    pub fn register_pane(&mut self, id: PaneId, element_count: usize) {
        let info = PaneInfo::new(id, element_count);
        self.panes.insert(id, info);
        
        if !self.tab_order.contains(&id) {
            self.tab_order.push(id);
        }
        
        if self.active_pane.is_none() {
            self.active_pane = Some(id);
            if let Some(pane) = self.panes.get_mut(&id) {
                pane.activate();
            }
        }
    }
    
    pub fn get_pane_info(&self, id: PaneId) -> Option<&PaneInfo> {
        self.panes.get(&id)
    }
    
    pub fn get_pane_info_mut(&mut self, id: PaneId) -> Option<&mut PaneInfo> {
        self.panes.get_mut(&id)
    }
    
    pub fn get_active_pane(&self) -> Option<&PaneInfo> {
        self.get_pane_info(self.active_pane?)
    }

    pub fn active_pane(&self) -> Option<PaneId> {
        self.active_pane
    }
    
    pub fn is_active(&self, id: PaneId) -> bool {
        self.active_pane == Some(id)
    }
    
    pub fn activate_pane(&mut self, id: PaneId) -> Result<()> {
        if !self.panes.contains_key(&id) {
            return Err(anyhow::anyhow!("Pane not registered: {:?}", id));
        }
        
        if let Some(active_id) = self.active_pane {
            if let Some(pane) = self.panes.get_mut(&active_id) {
                pane.deactivate();
            }
        }
        
        if let Some(pane) = self.panes.get_mut(&id) {
            pane.activate();
        }
        
        self.active_pane = Some(id);
        Ok(())
    }
    
    pub fn cycle_pane(&mut self, reverse: bool) -> Result<PaneId> {
        if self.tab_order.is_empty() {
            return Err(anyhow::anyhow!("No panes registered"));
        }
        
        let current_idx = if let Some(id) = self.active_pane {
            self.tab_order.iter().position(|&p| p == id).unwrap_or(0)
        } else {
            0
        };
        
        let next_idx = if reverse {
            if current_idx == 0 {
                self.tab_order.len() - 1
            } else {
                current_idx - 1
            }
        } else {
            (current_idx + 1) % self.tab_order.len()
        };
        
        let next_id = self.tab_order[next_idx];
        self.activate_pane(next_id)?;
        
        Ok(next_id)
    }
    
    pub fn start_editing(&mut self, id: PaneId) -> Result<()> {
        if !self.panes.contains_key(&id) {
            return Err(anyhow::anyhow!("Pane not registered: {:?}", id));
        }
        
        if self.active_pane != Some(id) {
            self.activate_pane(id)?;
        }
        
        if let Some(pane) = self.panes.get_mut(&id) {
            pane.start_editing();
        }
        
        Ok(())
    }
    
    pub fn stop_editing(&mut self, id: PaneId) -> Result<()> {
        if !self.panes.contains_key(&id) {
            return Err(anyhow::anyhow!("Pane not registered: {:?}", id));
        }
        
        if let Some(pane) = self.panes.get_mut(&id) {
            pane.stop_editing();
        }
        
        Ok(())
    }
    
    pub fn handle_tab(&mut self, reverse: bool) -> Result<(PaneId, bool)> {
        if let Some(id) = self.active_pane {
            if let Some(pane) = self.panes.get_mut(&id) {
                if pane.is_editing() && pane.element_count > 1 {
                    let handled = if reverse {
                        pane.prev_element()
                    } else {
                        pane.next_element()
                    };
                    
                    if handled {
                        return Ok((id, true));
                    }
                }
            }
        }
        
        let next_id = self.cycle_pane(reverse)?;
        Ok((next_id, false))
    }
    
    pub fn cycle_tab_order(&mut self, id: PaneId, new_position: usize) -> Result<()> {
        if !self.panes.contains_key(&id) {
            return Err(anyhow::anyhow!("Pane not registered: {:?}", id));
        }
        
        if let Some(pos) = self.tab_order.iter().position(|&p| p == id) {
            self.tab_order.remove(pos);
        }
        
        let pos = new_position.min(self.tab_order.len());
        self.tab_order.insert(pos, id);
        
        Ok(())
    }
}