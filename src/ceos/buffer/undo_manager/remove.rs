use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_possition::CaretPosition;
use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::buffer::undo_manager::insert::Insert;
use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::selection::Selection;
use std::fmt::{Display, Formatter};

/// A structure representing a `Remove` operation, typically used to denote
/// the deletion of a segment of text at a specific position within a document or editor.
///
/// # Fields
///
/// * `position` - The position where the text removal begins.
/// * `text` - The string content that was removed at the specified position.
#[derive(Debug)]
pub(crate) struct Remove {
    position: Position,
    lines: Vec<String>,
}

impl Remove {
    pub const fn new(position: Position, lines: Vec<String>) -> Self {
        Self { position, lines }
    }
}

impl Edit for Remove {
    fn undo(&self, buffer: &mut Buffer) -> CaretPosition {
        if self.lines.len() == 1 {
            buffer.insert_str(self.position, &self.lines[0]);
        } else {
            let suffix = buffer.filter_line_mut(self.position.line, |line| {
                let suffix = line.drain(self.position.column..).as_str().to_owned();
                line.push_str(&self.lines[0]);
                suffix
            });
            buffer.insert_lines(self.position.line + 1, self.lines[1..].into());
            if let Some(suffix) = suffix {
                buffer.filter_line_mut(self.position.line + self.lines.len() - 1, |line| {
                    line.push_str(&suffix);
                });
            }
        }

        CaretPosition::Selection(Selection::new(
            self.position,
            Position {
                line: self.position.line + self.lines.len(),
                column: self.lines.last().map(|line| line.len()).unwrap_or_default(),
            },
        ))
    }

    fn redo(&self, buffer: &mut Buffer) -> CaretPosition {
        buffer.delete_range(TextRange::new(
            self.position,
            Position::new(
                self.position.line + self.lines.len(),
                self.lines.last().map(|line| line.len()).unwrap_or_default(),
            ),
        ));
        CaretPosition::Position(self.position)
    }
}

#[cfg(test)]
impl Display for Remove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Remove {{ position: {:?}, lines: '{}' }}",
            self.position,
            self.lines.join("\n")
        )
    }
}
