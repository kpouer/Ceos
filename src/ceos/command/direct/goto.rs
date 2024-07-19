use crate::textarea::textareaproperties::TextAreaProperties;
use eframe::emath::Vec2;
use std::cmp;

pub(crate) fn execute(command: &str, textarea: &mut TextAreaProperties) {
    if let Some(stripped) = command.strip_prefix(':') {
        if let Ok(pos) = stripped.parse::<usize>() {
            let y_offset = textarea.line_height()
                * ((cmp::min(pos, textarea.buffer().line_count()) as f32) - 1.0);
            textarea.set_scroll_offset(Vec2::new(0.0, y_offset));
        }
    }
}
