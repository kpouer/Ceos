use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_state::CaretState;
use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::gui::textpane::position;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct Insert {
    /// The start position of the inserted lines.
    position: Position,
    /// The textrange that was inserted
    text_range: TextRange,
    lines: Vec<String>,
}

impl Insert {
    pub(crate) fn new(position: Position, lines: Vec<String>) -> Self {
        let end_position = if lines.len() == 1 {
            position.move_right_by(lines[0].len())
        } else {
            Position::new(
                position.line + lines.len() - 1,
                lines[lines.len() - 1].len(),
            )
        };
        let text_range = TextRange::new(position, end_position);
        Self {
            position,
            text_range,
            lines,
        }
    }

    pub(crate) fn try_merge(&mut self, other: &Self) -> bool {
        if self.text_range.end != other.position {
            return false;
        }

        if self.lines.len() == 1 && other.lines.len() == 1 {
            self.lines[0].push_str(&other.lines[0]);
            self.text_range.end = other.text_range.end;
            return true;
        }

        if self.lines.len() > 1 && other.lines.len() == 1 {
            self.lines
                .last_mut()
                .expect("Lines cannot be empty")
                .push_str(&other.lines[0]);
            self.text_range.end = other.text_range.end;
            return true;
        }

        if self.lines.len() == 1 && other.lines.len() > 1 {
            let other_lines = other.lines.clone();
            self.lines[0].push_str(&other_lines[0]);
            self.lines.extend(other_lines.iter().skip(1).cloned());
            self.text_range.end = other.text_range.end;
            return true;
        }

        if self.lines.len() > 1 && other.lines.len() > 1 {
            let other_lines = other.lines.clone();
            self.lines
                .last_mut()
                .expect("Lines cannot be empty")
                .push_str(&other_lines[0]);
            self.lines.extend(other_lines.iter().skip(1).cloned());
            self.text_range.end = other.text_range.end;
            return true;
        }

        false
    }

    pub(crate) fn undo(&self, buffer: &mut Buffer) -> CaretState {
        buffer.delete_range(self.text_range);
        CaretState::Position(self.position)
    }

    pub(crate) fn redo(&self, buffer: &mut Buffer) -> CaretState {
        match self.lines.as_slice() {
            [line] => {
                buffer.insert_str(self.position, line);
                CaretState::Position(self.position.move_right_by(line.len()))
            }
            [first, rest @ .., last] => {
                // Insère le texte avant le saut de ligne
                if !first.is_empty() {
                    buffer.insert_str(self.position, first);
                }
                buffer.insert_newline(self.position.move_right_by(first.len()));

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
                CaretState::Position(Position::new(
                    self.position.line + self.lines.len() - 1,
                    last.len(),
                ))
            }
            [] => CaretState::Position(self.position),
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
        if let CaretState::Position(p) = caret {
            assert_eq!(p, Position::new(0, 12));
        } else {
            panic!("Expected Position caret");
        }

        // Undo
        let caret = insert.undo(&mut buffer);
        assert_eq!(buffer.line_text(0), "Initial");
        if let CaretState::Position(p) = caret {
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

        if let CaretState::Position(p) = caret {
            assert_eq!(p, Position::new(2, 3));
        } else {
            panic!("Expected Position caret");
        }

        // Undo
        let caret = insert.undo(&mut buffer);
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line_text(0), "First");
        assert_eq!(buffer.line_text(1), "Last");
        if let CaretState::Position(p) = caret {
            assert_eq!(p, pos);
        } else {
            panic!("Expected Position caret");
        }
    }

    #[test]
    fn test_insert_merge() {
        let pos1 = Position::ZERO;
        let mut insert1 = Insert::new(pos1, vec!["Hello".to_string()]);
        let pos2 = Position::new(0, 5);
        let insert2 = Insert::new(pos2, vec![" World".to_string()]);

        assert!(insert1.try_merge(&insert2));
        assert_eq!(insert1.lines, vec!["Hello World".to_string()]);
        assert_eq!(insert1.text_range.start, Position::ZERO);
        assert_eq!(insert1.text_range.end, Position::new(0, 11));

        let pos3 = Position::new(0, 11);
        let insert3 = Insert::new(pos3, vec!["".to_string(), "New Line".to_string()]);
        assert!(insert1.try_merge(&insert3));
        assert_eq!(
            insert1.lines,
            vec!["Hello World".to_string(), "New Line".to_string()]
        );
        assert_eq!(insert1.text_range.end, Position::new(1, 8));
    }
}
