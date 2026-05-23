use std::fmt::Display;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub const ZERO: Position = Position { line: 0, column: 0 };

    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub const fn move_left(&self) -> Self {
        Self {
            line: self.line,
            column: self.column - 1,
        }
    }

    pub const fn move_right(&self) -> Self {
        Self {
            line: self.line,
            column: self.column + 1,
        }
    }

    pub const fn move_right_by(&self, amount: usize) -> Self {
        Self {
            line: self.line,
            column: self.column + amount,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, column: {}", self.line, self.column)
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
