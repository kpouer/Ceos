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
                Ok(Goto { line })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl Goto {
    pub(crate) fn execute(&self, textarea: &mut TextAreaProperties) {
        let y_offset = textarea.line_height()
            * ((cmp::min(self.line, textarea.buffer().line_count()) as f32) - 1.0);
        textarea.set_scroll_offset(Vec2::new(0.0, y_offset));
    }
}
