use crate::ceos::textarea::textareaproperties::TextAreaProperties;
use eframe::emath::Vec2;
use std::cmp;

pub(crate) struct Goto {
    line: usize,
}

impl TryFrom<&str> for Goto {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if let Some(stripped) = command.strip_prefix(':') {
            if let Ok(line) = stripped.parse::<usize>() {
                return Ok(Goto { line });
            }
        }
        Err(())
    }
}

impl Goto {
    pub(crate) fn execute(&self, textarea: &mut TextAreaProperties) {
        let y_offset = textarea.line_height()
            * ((cmp::min(self.line, textarea.buffer().line_count()) as f32) - 1.0);
        textarea.set_scroll_offset(Vec2::new(0.0, y_offset));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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
