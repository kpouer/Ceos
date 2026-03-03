use edit::Edit;

mod edit;

#[derive(Default, Debug)]
pub(crate) struct UndoManager {
    edits: Vec<Edit>,
}

impl UndoManager {
    pub(crate) fn push(&mut self, new_edit: Edit) {
        self.edits.push(new_edit);
    }
}
