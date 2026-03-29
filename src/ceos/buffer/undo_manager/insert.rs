use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_possition::CaretPosition;
use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct Insert {
    /// The start position of the inserted lines.
    position: Position,
    lines: Vec<String>,
}

impl Insert {
    pub(crate) const fn new(position: Position, lines: Vec<String>) -> Self {
        Self { position, lines }
    }
}

impl Edit for Insert {
    fn undo(&self, buffer: &mut Buffer) -> CaretPosition {
        let text_range = if self.lines.len() == 1 {
            TextRange::new(
                self.position,
                Position::new(
                    self.position.line,
                    self.position.column + self.lines[0].len(),
                ),
            )
        } else {
            TextRange::new(
                self.position,
                Position::new(
                    self.position.line + self.lines.len() - 1,
                    self.lines[self.lines.len() - 1].len(),
                ),
            )
        };
        buffer.delete_range(text_range);
        CaretPosition::Position(self.position)
    }

    fn redo(&self, buffer: &mut Buffer) -> CaretPosition {
        buffer.insert_str(self.position, &self.lines[0]);
        if self.lines.len() > 1 {
            buffer.insert_lines(self.position.line, self.lines[1..].to_owned());
        }

        if self.lines.len() == 1 {
            CaretPosition::Position(Position::new(
                self.position.line,
                self.position.column + self.lines[0].len(),
            ))
        } else {
            CaretPosition::Position(Position::new(
                self.position.line + self.lines.len() - 1,
                self.lines[self.lines.len() - 1].len(),
            ))
        }
    }

    #[cfg(test)]
    fn to_string(&self) -> String {
        format!(
            "Insert {{ position: {:?}, lines: {:?} }}",
            self.position, self.lines
        )
    }
}
