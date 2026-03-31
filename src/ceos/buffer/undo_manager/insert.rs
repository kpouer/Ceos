use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_possition::CaretPosition;
use crate::ceos::buffer::text_range::TextRange;
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
                buffer.insert_newline(Position::new(
                    self.position.line,
                    self.position.column + first.len(),
                ));

                // Insère les lignes suivantes
                if !rest.is_empty() {
                    buffer.insert_lines(self.position.line + 1, rest.to_vec());
                }

                // Insère le dernier segment et le suffixe de la ligne originale
                if !last.is_empty() {
                    buffer.insert_str(
                        Position::new(self.position.line + self.lines.len() - 1, 0),
                        last,
                    );
                }

                // Curseur à la fin du dernier segment inséré
                CaretPosition::Position(Position::new(
                    self.position.line + self.lines.len() - 1,
                    last.len(),
                ))
            }
            [] => CaretPosition::Position(self.position),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ceos::buffer::buffer::Buffer;
    use crate::ceos::gui::textpane::position::Position;

    #[test]
    fn test_insert_single_line() {
        let mut buffer = Buffer::new_test_buffer("Initial", 100);

        let pos = Position::new(0, 7);
        let insert = Insert::new(pos, vec![" text".to_string()]);

        // Redo
        let caret = insert.redo(&mut buffer);
        assert_eq!(buffer.line_text(0), "Initial text");
        if let CaretPosition::Position(p) = caret {
            assert_eq!(p, Position::new(0, 12));
        } else {
            panic!("Expected Position caret");
        }

        // Undo
        let caret = insert.undo(&mut buffer);
        assert_eq!(buffer.line_text(0), "Initial");
        if let CaretPosition::Position(p) = caret {
            assert_eq!(p, pos);
        } else {
            panic!("Expected Position caret");
        }
    }

    #[test]
    fn test_insert_multiple_lines() {
        let mut buffer = Buffer::new_test_buffer("First\nLast", 100);

        let pos = Position::new(0, 5);
        // Insert:
        // First\nMiddle\nEnd
        // results in:
        // First
        // Middle
        // EndLast
        let insert = Insert::new(
            pos,
            vec!["".to_string(), "Middle".to_string(), "End".to_string()],
        );

        // Redo
        let caret = insert.redo(&mut buffer);
        assert_eq!(buffer.line_count(), 4);
        assert_eq!(buffer.line_text(0), "First");
        assert_eq!(buffer.line_text(1), "Middle");
        assert_eq!(buffer.line_text(2), "End");
        assert_eq!(buffer.line_text(3), "Last");

        if let CaretPosition::Position(p) = caret {
            assert_eq!(p, Position::new(2, 3));
        } else {
            panic!("Expected Position caret");
        }

        // Undo
        let caret = insert.undo(&mut buffer);
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line_text(0), "First");
        assert_eq!(buffer.line_text(1), "Last");
        if let CaretPosition::Position(p) = caret {
            assert_eq!(p, pos);
        } else {
            panic!("Expected Position caret");
        }
    }
}
