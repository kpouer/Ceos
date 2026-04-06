use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_state::CaretState;
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
    /// The deleted text range.
    text_range: TextRange,
    lines: Vec<String>,
}

impl Remove {
    pub const fn new(position: Position, text_range: TextRange, lines: Vec<String>) -> Self {
        Self {
            position,
            text_range,
            lines,
        }
    }

    pub(crate) fn try_merge(&mut self, other: &Self) -> bool {
        // Suppression en arrière (backspace) : 'ba' -> 'b' puis 'a' supprimé.
        // On supprime d'abord 'a' à pos (0,2), puis 'b' à pos (0,1).
        // Donc 'self' est (0,2) et 'other' est (0,1).
        // other.text_range.end == self.position
        if other.text_range.end == self.position {
            let other_lines = other.lines.clone();
            if other_lines.len() == 1 && self.lines.len() == 1 {
                let mut new_lines = other_lines;
                new_lines[0].push_str(&self.lines[0]);
                self.lines = new_lines;
            } else if other_lines.len() > 1 && self.lines.len() == 1 {
                let mut new_lines = other_lines;
                new_lines
                    .last_mut()
                    .expect("Lines cannot be empty")
                    .push_str(&self.lines[0]);
                self.lines = new_lines;
            } else if other_lines.len() == 1 && self.lines.len() > 1 {
                let mut new_lines = other_lines;
                new_lines[0].push_str(&self.lines[0]);
                new_lines.extend(self.lines.iter().skip(1).cloned());
                self.lines = new_lines;
            } else {
                let mut new_lines = other_lines;
                new_lines
                    .last_mut()
                    .expect("Lines cannot be empty")
                    .push_str(&self.lines[0]);
                new_lines.extend(self.lines.iter().skip(1).cloned());
                self.lines = new_lines;
            }
            self.position = other.position;
            self.text_range.start = other.text_range.start;
            return true;
        }

        // Suppression en avant (delete) : 'ab' -> 'a' puis 'b' supprimé.
        // On supprime d'abord 'a' à pos (0,1), puis 'b' à pos (0,1).
        // Donc self.position == other.position
        if self.position == other.position {
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
                self.lines[0].push_str(&other.lines[0]);
                self.lines.extend(other.lines.iter().skip(1).cloned());
                self.text_range.end = other.text_range.end;
                return true;
            }

            if self.lines.len() > 1 && other.lines.len() > 1 {
                self.lines
                    .last_mut()
                    .expect("Lines cannot be empty")
                    .push_str(&other.lines[0]);
                self.lines.extend(other.lines.iter().skip(1).cloned());
                self.text_range.end = other.text_range.end;
                return true;
            }
        }

        false
    }

    pub(crate) fn undo(&self, buffer: &mut Buffer) -> CaretState {
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

        CaretState::Selection(self.text_range.into())
    }

    pub(crate) fn redo(&self, buffer: &mut Buffer) -> CaretState {
        buffer.delete_range(self.text_range);
        CaretState::Position(self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ceos::buffer::buffer::Buffer;
    use crate::ceos::buffer::caret_state::CaretState;
    use crate::ceos::gui::textpane::position::Position;

    #[test]
    fn test_remove_single_line() {
        let mut buffer = Buffer::new_test_buffer("Initial text", 100);

        let pos = Position::new(0, 7);
        let text_range = TextRange::new(pos, Position::new(0, 12));
        // " text" has length 5
        let remove = Remove::new(pos, text_range, vec![" text".to_string()]);

        // Redo
        let caret = remove.redo(&mut buffer);
        assert_eq!(buffer.line_text(0), "Initial");
        if let CaretState::Position(p) = caret {
            assert_eq!(p, pos);
        } else {
            panic!("Expected Position caret");
        }

        // Undo
        let caret = remove.undo(&mut buffer);
        assert_eq!(buffer.line_text(0), "Initial text");
        if let CaretState::Selection(s) = caret {
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
        let text_range = TextRange::new(pos, Position::new(2, 3));
        let remove = Remove::new(
            pos,
            text_range,
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
        if let CaretState::Position(p) = caret {
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

        if let CaretState::Selection(s) = caret {
            assert_eq!(s.start, pos);
            assert_eq!(s.end, Position::new(2, 3));
        } else {
            panic!("Expected Selection caret");
        }
    }

    #[test]
    fn test_remove_merge_backspace() {
        // "Hello World" -> "Hello " (on supprime 'W', 'o', 'r', 'l', 'd' un par un en arrière)
        let pos1 = Position::new(0, 10); // supprime 'd' à la fin de "Hello World"
        let range1 = TextRange::new(pos1, Position::new(0, 11));
        let mut remove1 = Remove::new(pos1, range1, vec!["d".to_string()]);

        let pos2 = Position::new(0, 9); // supprime 'l'
        let range2 = TextRange::new(pos2, Position::new(0, 10));
        let remove2 = Remove::new(pos2, range2, vec!["l".to_string()]);

        assert!(remove1.try_merge(&remove2));
        assert_eq!(remove1.lines, vec!["ld".to_string()]);
        assert_eq!(remove1.position, Position::new(0, 9));
        assert_eq!(remove1.text_range.start, Position::new(0, 9));
        assert_eq!(remove1.text_range.end, Position::new(0, 11));
    }

    #[test]
    fn test_remove_merge_delete() {
        // "Hello World" -> "Hello " (on supprime 'W', 'o', 'r', 'l', 'd' un par un en avant)
        let pos1 = Position::new(0, 6); // supprime 'W'
        let range1 = TextRange::new(pos1, Position::new(0, 7));
        let mut remove1 = Remove::new(pos1, range1, vec!["W".to_string()]);

        let pos2 = Position::new(0, 6); // supprime 'o' (qui a pris la place de 'W')
        let range2 = TextRange::new(pos2, Position::new(0, 7));
        let remove2 = Remove::new(pos2, range2, vec!["o".to_string()]);

        assert!(remove1.try_merge(&remove2));
        assert_eq!(remove1.lines, vec!["Wo".to_string()]);
        assert_eq!(remove1.position, Position::new(0, 6));
        assert_eq!(remove1.text_range.start, Position::new(0, 6));
        assert_eq!(remove1.text_range.end, Position::new(0, 7));
    }
}
