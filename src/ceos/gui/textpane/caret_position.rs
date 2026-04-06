use crate::ceos::gui::textpane::position::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub(crate) struct CaretPosition {
    /// The caret position in the buffer.
    pub(crate) position: Position,
    /// the column where the caret would like to be (but maybe the line is to short).
    pub(crate) virtual_column: usize,
}

impl CaretPosition {
    pub(crate) const ZERO: CaretPosition = CaretPosition {
        position: Position::ZERO,
        virtual_column: 0,
    };

    pub(crate) const fn reset_virtual_column(&mut self) {
        self.virtual_column = self.position.column;
    }

    #[cfg(test)]
    pub(crate) const fn from_position(position: Position) -> Self {
        Self {
            position,
            virtual_column: position.column,
        }
    }
}
