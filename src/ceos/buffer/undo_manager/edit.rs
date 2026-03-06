use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::gui::textpane::position::Position;
use std::fmt::Debug;

pub(crate) trait Edit: Debug + Send + Sync {
    fn undo(&self, buffer: &mut Buffer) -> Position;
    fn redo(&self, buffer: &mut Buffer) -> Position;
}

#[derive(Debug)]
pub(crate) struct RemoveRange {
    line: usize,
    offset: usize,
    text: String,
}

impl RemoveRange {
    pub(crate) fn new(line: usize, offset: usize, text: String) -> Self {
        Self { line, offset, text }
    }
}

impl Edit for RemoveRange {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.insert_str(self.line, self.offset, &self.text);
        Position {
            line: self.line,
            column: self.offset,
        }
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct Insert {
    line: usize,
    start: usize,
    text: String,
}

impl Insert {
    pub(crate) fn new(line: usize, start: usize, text: String) -> Self {
        Self { line, start, text }
    }
}

impl Edit for Insert {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.delete_line_range(self.line, self.start, self.text.len());
        Position {
            line: self.line,
            column: self.start,
        }
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct Replace {
    line: usize,
    start: usize,
    end: usize,
    text: String,
}

impl Replace {
    pub(crate) fn new(line: usize, start: usize, end: usize, text: String) -> Self {
        Self {
            line,
            start,
            end,
            text,
        }
    }
}

impl Edit for Replace {
    fn undo(&self, _buffer: &mut Buffer) -> Position {
        // To undo a replacement, we would need the original text.
        // Assuming Replace here means we replaced [start, end) with `text`.
        // If we don't have the original text, we can't fully undo it.
        // But looking at Buffer, it seems Replace is not yet used for pushing to UndoManager.
        Position::default()
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct InsertText {
    line: usize,
    offset: usize,
    length: usize,
}

impl InsertText {
    pub(crate) fn new(line: usize, offset: usize, length: usize) -> Self {
        Self {
            line,
            offset,
            length,
        }
    }
}

impl Edit for InsertText {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.delete_line_range(self.line, self.offset, self.length);
        Position {
            line: self.line,
            column: self.offset,
        }
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct CompoundEdit {
    edits: Vec<Box<dyn Edit>>,
}

impl CompoundEdit {
    pub(crate) fn new(edits: Vec<Box<dyn Edit>>) -> Self {
        Self { edits }
    }
}

impl Edit for CompoundEdit {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        let mut pos = Position::default();
        for e in self.edits.iter().rev() {
            pos = e.undo(buffer);
        }
        pos
    }

    fn redo(&self, buffer: &mut Buffer) -> Position {
        let mut pos = Position::default();
        for e in self.edits.iter() {
            pos = e.redo(buffer);
        }
        pos
    }
}

#[derive(Debug)]
pub(crate) struct RemoveLines {
    lines: Vec<String>,
    line: usize,
}

impl RemoveLines {
    pub(crate) fn new(lines: Vec<String>, line: usize) -> Self {
        Self { lines, line }
    }
}

impl Edit for RemoveLines {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.insert_lines(self.line, self.lines.clone());
        Position {
            line: self.line,
            column: 0,
        }
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}
