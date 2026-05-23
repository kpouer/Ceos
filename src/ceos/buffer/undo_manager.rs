use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_state::CaretState;
use crate::ceos::buffer::undo_manager::insert::Insert;
use crate::ceos::buffer::undo_manager::remove::Remove;
use log::debug;

pub mod insert;
pub mod remove;

#[derive(Default, Debug)]
pub struct UndoManager {
    undos: Vec<UndoOperation>,
    redos: Vec<UndoOperation>,
    operation_in_progress: bool,
}

impl UndoManager {
    pub fn push_undo(&mut self, new_edit: UndoOperation, clear_redo: bool) {
        if self.operation_in_progress {
            return;
        }
        debug!("Pushing edit: {new_edit:?}");

        if let Some(last) = self.undos.last_mut()
            && last.try_merge(&new_edit)
        {
            if clear_redo {
                self.redos.clear();
            }
            return;
        }

        self.undos.push(new_edit);
        if clear_redo {
            self.redos.clear();
        }
    }

    pub fn push_redo(&mut self, edit: UndoOperation) {
        if self.operation_in_progress {
            return;
        }
        self.redos.push(edit);
    }

    pub fn pop_undo(&mut self) -> Option<UndoOperation> {
        self.undos.pop()
    }

    pub fn pop_redo(&mut self) -> Option<UndoOperation> {
        self.redos.pop()
    }

    pub const fn can_undo(&self) -> bool {
        !self.undos.is_empty()
    }

    pub const fn start_operation(&mut self) {
        assert!(!self.operation_in_progress);
        self.operation_in_progress = true;
    }

    pub const fn end_operation(&mut self) {
        assert!(self.operation_in_progress);
        self.operation_in_progress = false;
    }

    #[cfg(test)]
    pub fn last_undo(&self) -> Option<&UndoOperation> {
        self.undos.last()
    }
}

#[derive(Debug)]
pub enum UndoOperation {
    Insert(Insert),
    Remove(Remove),
}

#[cfg(test)]
impl std::fmt::Display for UndoOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UndoOperation::Insert(insert) => write!(f, "{insert:?}"),
            UndoOperation::Remove(remove) => write!(f, "{remove:?}"),
        }
    }
}

impl UndoOperation {
    pub fn try_merge(&mut self, other: &Self) -> bool {
        match (self, other) {
            (UndoOperation::Insert(this), UndoOperation::Insert(other)) => this.try_merge(other),
            (UndoOperation::Remove(this), UndoOperation::Remove(other)) => this.try_merge(other),
            _ => false,
        }
    }

    pub fn undo(&self, buffer: &mut Buffer) -> CaretState {
        match self {
            UndoOperation::Insert(insert) => insert.undo(buffer),
            UndoOperation::Remove(remove) => remove.undo(buffer),
        }
    }

    pub fn redo(&self, buffer: &mut Buffer) -> CaretState {
        match self {
            UndoOperation::Insert(insert) => insert.redo(buffer),
            UndoOperation::Remove(remove) => remove.redo(buffer),
        }
    }
}
