use edit::Edit;
use log::debug;

pub(crate) mod compound_edit;
pub(crate) mod edit;
pub(crate) mod insert_new_line;
pub(crate) mod insert_text;
pub(crate) mod remove_lines;
pub(crate) mod remove_range;
pub(crate) mod replace;

#[derive(Default, Debug)]
pub(crate) struct UndoManager {
    edits: Vec<Box<dyn Edit>>,
}

impl UndoManager {
    pub(crate) fn push(&mut self, new_edit: Box<dyn Edit>) {
        debug!("Pushing edit: {new_edit:?}");
        self.edits.push(new_edit);
    }

    pub(crate) fn pop(&mut self) -> Option<Box<dyn Edit>> {
        self.edits.pop()
    }

    pub(crate) const fn can_undo(&self) -> bool {
        !self.edits.is_empty()
    }
}
