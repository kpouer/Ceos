use crate::ceos::buffer::Buffer;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use eframe::emath::{Pos2, Rect};
use eframe::epaint::Stroke;
use egui::Ui;
use std::fmt::Display;

/// Search filter
pub(crate) struct Search {
    pattern: String,
    // the lines containing the search value
    pub(crate) lines: Vec<usize>,
}

impl TryFrom<&str> for Search {
    type Error = String;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if command.starts_with("s ") && command.len() > 2 {
            let pattern = command[2..].to_string();
            Ok(Self {
                pattern,
                lines: Vec::new(),
            })
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

impl Search {
    pub(crate) fn init(&mut self, buffer: &Buffer) {
        buffer.content.iter().enumerate().for_each(|(i, line)| {
            if line.content.contains(&self.pattern) {
                self.lines.push(i);
            }
        })
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
