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
}

impl UndoManager {
    pub(crate) fn push(&mut self, new_edit: Box<dyn Edit>) {
        debug!("Pushing edit: {new_edit:?}");
        self.undos.push(new_edit);
        self.redos.clear();
    }

    pub(crate) fn push_redo(&mut self, edit: Box<dyn Edit>) {
        self.redos.push(edit);
    }

    pub(crate) fn pop_undo(&mut self) -> Option<Box<dyn Edit>> {
        self.undos.pop()
    }

    pub(crate) fn pop_redo(&mut self) -> Option<Box<dyn Edit>> {
        self.redos.pop()
    }

    pub(crate) const fn can_undo(&self) -> bool {
        !self.undos.is_empty()
    }

    pub(crate) const fn can_redo(&self) -> bool {
        !self.redos.is_empty()
    }

    pub(crate) fn undo(&mut self, buffer: &mut Buffer) -> Option<CaretPosition> {
        if let Some(undo) = self.undos.pop() {
            let selection = undo.undo(buffer);
            self.redos.push(undo);
            Some(selection)
        } else {
            None
        }
    }

    pub(crate) fn redo(&mut self, buffer: &mut Buffer) -> Option<CaretPosition> {
        if let Some(redo) = self.redos.pop() {
            let selection = redo.redo(buffer);
            self.undos.push(redo);
            Some(selection)
        } else {
            None
        }
    }
}

#[cfg(test)]
impl UndoManager {
    pub(crate) fn last_undo(&self) -> Option<&Box<dyn Edit>> {
        self.undos.last()
    }
}
