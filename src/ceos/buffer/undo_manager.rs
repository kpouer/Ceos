use edit::Edit;
use log::debug;

pub(crate) mod compound_edit;
pub(crate) mod edit;
pub(crate) mod insert_text;
pub(crate) mod remove_lines;
pub(crate) mod remove_range;

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

    pub(crate) fn pop_undo(&mut self) -> Option<Box<dyn Edit>> {
        let edit = self.undos.pop();
        if let Some(ref e) = edit {
            debug!("Popping undo edit: {e:?}");
        }
        edit
    }

    pub(crate) fn push_redo(&mut self, edit: Box<dyn Edit>) {
        self.redos.push(edit);
    }

    pub(crate) fn pop_redo(&mut self) -> Option<Box<dyn Edit>> {
        let edit = self.redos.pop();
        if let Some(ref e) = edit {
            debug!("Popping redo edit: {e:?}");
        }
        edit
    }

    pub(crate) fn push_undo(&mut self, edit: Box<dyn Edit>) {
        self.undos.push(edit);
    }

    pub(crate) const fn can_undo(&self) -> bool {
        !self.undos.is_empty()
    }

    pub(crate) const fn can_redo(&self) -> bool {
        !self.redos.is_empty()
    }
}
