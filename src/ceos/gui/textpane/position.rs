use std::fmt::Display;

#[derive(Default, Debug)]
pub(crate) struct Position {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("line: {}, column: {}", self.line, self.column))
    }
}