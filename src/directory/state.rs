#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionState {
    Included,
    Excluded,
    Partial, // Some children included, some excluded
}

impl Default for SelectionState {
    fn default() -> Self {
        SelectionState::Excluded
    }
}

impl SelectionState {
    pub fn is_included(&self) -> bool {
        matches!(self, SelectionState::Included | SelectionState::Partial)
    }

    pub fn is_excluded(&self) -> bool {
        matches!(self, SelectionState::Excluded)
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, SelectionState::Partial)
    }

    pub fn toggle(&self) -> Self {
        match self {
            SelectionState::Included => SelectionState::Excluded,
            SelectionState::Excluded => SelectionState::Included,
            SelectionState::Partial => SelectionState::Included,
        }
    }
}