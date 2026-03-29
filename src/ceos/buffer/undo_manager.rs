use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_possition::CaretPosition;
use edit::Edit;
use log::debug;

pub(crate) mod edit;
pub(crate) mod insert;
pub(crate) mod remove;

#[derive(Default, Debug)]
pub(crate) struct UndoManager {
    undos: Vec<Box<dyn Edit>>,
    redos: Vec<Box<dyn Edit>>,
    operation_in_progress: bool,
}

impl UndoManager {
    pub(crate) fn push_undo(&mut self, new_edit: Box<dyn Edit>, clear_redo: bool) {
        if self.operation_in_progress {
            return;
        }
        debug!("Pushing edit: {new_edit:?}");
        self.undos.push(new_edit);
        if clear_redo {
            self.redos.clear();
        }
    }

    pub(crate) fn push_redo(&mut self, edit: Box<dyn Edit>) {
        if self.operation_in_progress {
            return;
        }
        self.redos.push(edit);
    }

    pub(crate) fn pop_undo(&mut self) -> Option<Box<dyn Edit>> {
        self.undos.pop()
    }

    pub(crate) fn pop_redo(&mut self) -> Option<Box<dyn Edit>> {
        self.redos.pop()
    }

    pub(crate) const fn is_operation_in_progress(&self) -> bool {
        self.operation_in_progress
    }

    pub(crate) const fn can_undo(&self) -> bool {
        !self.undos.is_empty()
    }

    pub(crate) const fn can_redo(&self) -> bool {
        !self.redos.is_empty()
    }

    pub(crate) fn clear_redo(&mut self) {
        self.redos.clear();
    }

    pub(crate) const fn start_operation(&mut self) {
        assert!(!self.operation_in_progress);
        self.operation_in_progress = true;
    }

    pub(crate) const fn end_operation(&mut self) {
        assert!(self.operation_in_progress);
        self.operation_in_progress = false;
    }
}

#[cfg(test)]
impl UndoManager {
    pub(crate) fn last_undo(&self) -> Option<&Box<dyn Edit>> {
        self.undos.last()
    }
}
