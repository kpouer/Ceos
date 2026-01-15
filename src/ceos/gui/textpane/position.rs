use std::fmt::Display;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Position {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("line: {}, column: {}", self.line, self.column))
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.line < other.line {
            Some(std::cmp::Ordering::Less)
        } else if self.line == other.line {
            Some(self.column.cmp(&other.column))
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}