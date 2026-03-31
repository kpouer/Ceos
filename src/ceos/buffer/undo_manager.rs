use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_possition::CaretPosition;
use crate::ceos::buffer::undo_manager::insert::Insert;
use crate::ceos::buffer::undo_manager::remove::Remove;
use log::debug;

pub(crate) mod insert;
pub(crate) mod remove;

#[derive(Default, Debug)]
pub(crate) struct UndoManager {
    undos: Vec<UndoOperation>,
    redos: Vec<UndoOperation>,
    operation_in_progress: bool,
}

impl UndoManager {
    pub(crate) fn push_undo(&mut self, new_edit: UndoOperation, clear_redo: bool) {
        if self.operation_in_progress {
            return;
        }
        debug!("Pushing edit: {new_edit:?}");
        self.undos.push(new_edit);
        if clear_redo {
            self.redos.clear();
        }
    }

    pub(crate) fn push_redo(&mut self, edit: UndoOperation) {
        if self.operation_in_progress {
            return;
        }
        self.redos.push(edit);
    }

    pub(crate) fn pop_undo(&mut self) -> Option<UndoOperation> {
        self.undos.pop()
    }

    pub(crate) fn pop_redo(&mut self) -> Option<UndoOperation> {
        self.redos.pop()
    }

    pub(crate) const fn can_undo(&self) -> bool {
        !self.undos.is_empty()
    }

    pub(crate) const fn can_redo(&self) -> bool {
        !self.redos.is_empty()
    }

    pub(crate) const fn start_operation(&mut self) {
        assert!(!self.operation_in_progress);
        self.operation_in_progress = true;
    }

    pub(crate) const fn end_operation(&mut self) {
        assert!(self.operation_in_progress);
        self.operation_in_progress = false;
    }

    #[cfg(test)]
    pub(crate) fn last_undo(&self) -> Option<&UndoOperation> {
        self.undos.last()
    }
}

#[derive(Debug)]
pub(crate) enum UndoOperation {
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
    pub(crate) fn undo(&self, buffer: &mut Buffer) -> CaretPosition {
        match self {
            UndoOperation::Insert(insert) => insert.undo(buffer),
            UndoOperation::Remove(remove) => remove.undo(buffer),
        }
    }

    pub(crate) fn redo(&self, buffer: &mut Buffer) -> CaretPosition {
        match self {
            UndoOperation::Insert(insert) => insert.redo(buffer),
            UndoOperation::Remove(remove) => remove.redo(buffer),
        }
    }
}
