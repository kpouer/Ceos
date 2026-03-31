use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_possition::CaretPosition;
use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::selection::Selection;

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

    pub(crate) fn undo(&self, buffer: &mut Buffer) -> CaretPosition {
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
                line: self.position.line + self.lines.len() - 1,
                column: if self.lines.len() == 1 {
                    self.position.column + self.lines[0].len()
                } else {
                    self.lines.last().map(|line| line.len()).unwrap_or_default()
                },
            },
        ))
    }

    pub(crate) fn redo(&self, buffer: &mut Buffer) -> CaretPosition {
        buffer.delete_range(TextRange::new(
            self.position,
            Position::new(
                self.position.line + self.lines.len() - 1,
                if self.lines.len() == 1 {
                    self.position.column + self.lines[0].len()
                } else {
                    self.lines.last().map(|line| line.len()).unwrap_or_default()
                },
            ),
        ));
        CaretPosition::Position(self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ceos::buffer::buffer::Buffer;
    use crate::ceos::buffer::caret_possition::CaretPosition;
    use crate::ceos::gui::textpane::position::Position;

    #[test]
    fn test_remove_single_line() {
        let mut buffer = Buffer::new_test_buffer("Initial text", 100);

        let pos = Position::new(0, 7);
        // " text" has length 5
        let remove = Remove::new(pos, vec![" text".to_string()]);

        // Redo
        let caret = remove.redo(&mut buffer);
        assert_eq!(buffer.line_text(0), "Initial");
        if let CaretPosition::Position(p) = caret {
            assert_eq!(p, pos);
        } else {
            panic!("Expected Position caret");
        }

        // Undo
        let caret = remove.undo(&mut buffer);
        assert_eq!(buffer.line_text(0), "Initial text");
        if let CaretPosition::Selection(s) = caret {
            assert_eq!(s.start, pos);
            assert_eq!(s.end, Position::new(0, 12));
        } else {
            panic!("Expected Selection caret");
        }
    }

    #[test]
    fn test_remove_multiple_lines() {
        // Initial state:
        // Line 0: "First"
        // Line 1: "Middle"
        // Line 2: "EndLast"
        let mut buffer = Buffer::new_test_buffer("First\nMiddle\nEndLast", 100);

        let pos = Position::new(0, 5);
        // lines removed:
        // line 0: suffix ""
        // line 1: "Middle"
        // line 2: "End"
        let remove = Remove::new(
            pos,
            vec!["".to_string(), "Middle".to_string(), "End".to_string()],
        );

        // Redo
        let caret = remove.redo(&mut buffer);

        // Analyse du résultat attendu de delete_range((0,5) -> (2,3))
        // Ligne 0: "First" -> reste "First"
        // Ligne 2: "EndLast" -> reste "Last"
        // Fusion des deux : "FirstLast" sur la ligne 0.

        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.line_text(0), "FirstLast");
        if let CaretPosition::Position(p) = caret {
            assert_eq!(p, pos);
        } else {
            panic!("Expected Position caret");
        }

        // Undo
        let caret = remove.undo(&mut buffer);
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.line_text(0), "First");
        assert_eq!(buffer.line_text(1), "Middle");
        assert_eq!(buffer.line_text(2), "EndLast");

        if let CaretPosition::Selection(s) = caret {
            assert_eq!(s.start, pos);
            assert_eq!(s.end, Position::new(2, 3));
        } else {
            panic!("Expected Selection caret");
        }
    }
}
