use eframe::emath::{Pos2, Rect};
use eframe::epaint::Stroke;
use egui::Ui;
use log::info;
use std::fmt::Display;

use crate::ceos::buffer::Buffer;
use crate::ceos::command::Command;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;

pub(crate) struct Search {
    pattern: String,
}

impl TryFrom<&str> for Search {
    type Error = String;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if command.starts_with("s ") && command.len() > 2 {
            let pattern = command[2..].to_string();
            Ok(Self { pattern })
        } else {
            Err("Command not valid".to_string())
        }
    }
}

impl Renderer for Search {
    fn paint_line(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        textarea: &TextAreaProperties,
        line: usize,
        drawing_pos: Pos2,
    ) {
        let line = &textarea.buffer.content[line];
        if let Some(offset) = line.content.find(&self.pattern) {
            let x1 = offset as f32 * textarea.char_width;
            let x2 = (offset + self.pattern.len()) as f32 * textarea.char_width;
            let top_left = Pos2::new(drawing_pos.x + x1, drawing_pos.y);
            let bottom_right = Pos2::new(drawing_pos.x + x2, drawing_pos.y + textarea.line_height);
            let line_rect = Rect::from_min_max(top_left, bottom_right);
            let painter = ui.painter();
            painter.rect(line_rect, 0.0, theme.deleting, Stroke::default());
        }
    }
}

impl Command for Search {
    fn execute(&self, buffer: &mut Buffer) {
        let line_count = buffer.line_count();
        // buffer.content.retain(|line| self.accept(line));
        //
        let new_length = buffer.compute_length();
        info!(
            "Applied filter '{}' removed {} lines, new length {new_length}",
            self.pattern,
            line_count - buffer.line_count()
        );
    }
}

impl Display for Search {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Search '{}'", self.pattern)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_filter() -> anyhow::Result<(), String> {
//         let filter = LineFilter::try_from("filter delete")?;
//         let content = "1 delete me\n\
//         2 keep me\n\
//         3 delete me\n\
//         4 keep me\n";
//         let mut buffer = Buffer::from(content);
//         assert_eq!(content.len(), buffer.len());
//         assert_eq!(4, buffer.line_count());
//         filter.execute(&mut buffer);
//         assert_eq!(2, buffer.line_count());
//         Ok(())
//     }
// }
