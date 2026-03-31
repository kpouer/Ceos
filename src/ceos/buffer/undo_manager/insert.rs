use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_possition::CaretPosition;
use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::gui::textpane::position::Position;
use std::fmt::{Display, Formatter};

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

    pub(crate) fn undo(&self, buffer: &mut Buffer) -> CaretPosition {
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

    pub(crate) fn redo(&self, buffer: &mut Buffer) -> CaretPosition {
        match self.lines.as_slice() {
            [line] => {
                buffer.insert_str(self.position, line);
                CaretPosition::Position(Position::new(
                    self.position.line,
                    self.position.column + line.len(),
                ))
            }
            [first, rest @ .., last] => {
                // Insère le texte avant le saut de ligne
                if !first.is_empty() {
                    buffer.insert_str(self.position, first);
                }
                buffer.insert_newline(self.position);

                // Insère les lignes suivantes
                if !rest.is_empty() {
                    buffer.insert_lines(self.position.line + 1, rest.to_vec());
                }

                // Curseur à la fin du dernier segment inséré
                CaretPosition::Position(Position::new(
                    self.position.line + self.lines.len() - 1,
                    self.lines.last().map(|s| s.len()).unwrap_or(0),
                ))
            }
            [] => CaretPosition::Position(self.position),
        }
    }
}

#[cfg(test)]
impl Display for Insert {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Insert {{ position: {:?}, lines: {:?} }}",
            self.position, self.lines
        )
    }
}
