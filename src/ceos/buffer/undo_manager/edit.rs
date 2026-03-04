#[derive(Debug)]
pub(crate) enum Edit {
    RemoveRange {
        offset: usize,
        text: String,
    },
    Insert {
        start: usize,
        text: String,
    },
    Replace {
        start: usize,
        end: usize,
        text: String,
    },
    InsertText {
        offset: usize,
        length: usize,
    },
    CompoundEdit {
        edits: Vec<Edit>,
    },
    RemoveLines {
        lines: Vec<String>,
        line: usize,
    },
}
