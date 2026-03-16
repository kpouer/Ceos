use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_possition::CaretPosition;
use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct Insert {
    position: Position,
    lines: Vec<String>,
}

impl Insert {
    pub fn new(position: Position, lines: Vec<String>) -> Self {
        Self { position, lines }
    }
}

impl Edit for Insert {
    fn undo(&self, buffer: &mut Buffer) -> CaretPosition {
        let last_line_pos = self.lines.last().map(|s| s.len()).unwrap_or(0);
        buffer.delete_range(TextRange::new(
            Position::new(self.position.line, self.position.column),
            Position::new(self.position.line + self.lines.len(), last_line_pos),
        ));
        CaretPosition::Position(self.position)
    }

    fn redo(&self, buffer: &mut Buffer) -> CaretPosition {
        todo!()
    }

    #[cfg(test)]
    fn to_string(&self) -> String {
        format!(
            "Insert {{ position: {:?}, lines: {:?} }}",
            self.position, self.lines
        )
    }
}
