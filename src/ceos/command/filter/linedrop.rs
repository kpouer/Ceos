use std::cmp;
use std::fmt::Display;

use eframe::emath::{Pos2, Rect};
use eframe::epaint::Stroke;
use egui::Ui;
use log::info;

use crate::ceos::command::Command;
use crate::ceos::buffer::Buffer;
use crate::ceos::gui::textarea::renderer::Renderer;
use crate::ceos::gui::textarea::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::tools::range::Range;

#[derive(Debug, PartialEq)]
pub(crate) struct LineDrop {
    range: Range,
}

impl TryFrom<&str> for LineDrop {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(remaining) = value.strip_prefix("l ") {
            return Ok(LineDrop {
                range: Range::try_from(remaining)?,
            });
        }
        Err(())
    }
}

impl Renderer for LineDrop {
    fn paint_line(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        textarea: &TextAreaProperties,
        line: usize,
        _: Pos2,
        drawing_pos: Pos2,
    ) {
        if self.range.contains(line + 1) {
            let bottom_right =
                Pos2::new(ui.max_rect().max.x, drawing_pos.y + textarea.line_height());
            let line_rect = Rect::from_min_max(drawing_pos, bottom_right);
            let painter = ui.painter();
            painter.rect(line_rect, 0.0, theme.deleting, Stroke::default());
        }
    }
}

impl Command for LineDrop {
    fn execute(&self, buffer: &mut Buffer) {
        let line_count = buffer.content().len();
        if let Some(end) = self.range.end {
            buffer
                .content_mut()
                .drain(self.range.start..cmp::min(line_count, end));
        } else {
            buffer.content_mut().drain(self.range.start..);
        }

        let new_length = buffer.compute_length();
        info!(
            "Removed range '{:?}' removed {} lines, new length {new_length}",
            self.range,
            line_count - buffer.content().len()
        );
    }
}

impl Display for LineDrop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LineDrop '{}:{:?}'", self.range.start, self.range.end)
    }
}
