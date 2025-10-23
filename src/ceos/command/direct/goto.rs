use std::cmp;

use eframe::emath::Vec2;
use egui::Context;

use crate::ceos::gui::textpane::TextPaneState;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;

#[derive(Debug)]
pub(crate) struct Goto {
    line: usize,
}

impl From<usize> for Goto {
    fn from(line: usize) -> Self {
        Self { line }
    }
}

impl TryFrom<&str> for Goto {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if let Some(stripped) = command.strip_prefix(':')
            && let Ok(line) = stripped.parse::<usize>()
        {
            return Ok(Goto { line });
        }
        Err(())
    }
}

impl Goto {
    pub(crate) fn new(line: usize) -> Self {
        Self { line }
    }

    pub(crate) fn execute(&self, ctx: &Context, textarea: &mut TextAreaProperties) {
        let y_offset = textarea.line_height
            * ((cmp::min(self.line, textarea.buffer.line_count()) as f32) - 1.0);
        let mut textpane_state = TextPaneState::get(ctx);
        textpane_state.scroll_offset = Vec2::new(0.0, y_offset);
        ctx.memory_mut(|m| m.data.insert_temp(TextPaneState::id(), textpane_state));
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(3, ":3")]
    #[case(333, ":333")]
    fn test_try_from_valid_command(#[case] expected: usize, #[case] command: &str) {
        let goto = Goto::try_from(command).unwrap();
        assert_eq!(expected, goto.line);
    }

    #[rstest]
    #[case("invalid")]
    #[case(":3inv")]
    #[case(":")]
    #[case(":bubu")]
    #[case(":-1")]
    fn test_try_from_invalid_command(#[case] command: &str) {
        let goto = Goto::try_from(command);
        assert!(goto.is_err());
    }
}
