use edit::Edit;
use log::debug;

pub(crate) mod edit;

#[derive(Default, Debug)]
pub(crate) struct UndoManager {
    edits: Vec<Edit>,
}

impl UndoManager {
    pub(crate) fn push(&mut self, new_edit: Edit) {
        debug!("Pushing edit: {new_edit:?}");
        self.edits.push(new_edit);
    }
}
