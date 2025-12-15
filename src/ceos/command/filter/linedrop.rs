use std::cmp;
use std::fmt::Display;

use eframe::emath::{Pos2, Rect};
use eframe::epaint::Stroke;
use egui::{StrokeKind, Ui};
use log::info;

use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::command::Command;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::tools::range::Range;

/// LineDrop filter
///
/// It will drop a range of lines of the buffer
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
        drawing_pos: Pos2,
    ) {
        if self.range.contains(line + 1) {
            let bottom_right = Pos2::new(ui.max_rect().max.x, drawing_pos.y + textarea.line_height);
            let line_rect = Rect::from_min_max(drawing_pos, bottom_right);
            let painter = ui.painter();
            painter.rect(
                line_rect,
                0.0,
                theme.deleting,
                Stroke::default(),
                StrokeKind::Inside,
            );
        }
    }
}

impl Command for LineDrop {
    fn execute(&self, buffer: &mut Buffer) {
        let line_count = buffer.line_count();
        let new_length = if let Some(end) = self.range.end {
            buffer.drain_line_mut(self.range.start..cmp::min(line_count, end))
        } else {
            buffer.drain_line_mut(self.range.start..)
        };

        info!(
            "Removed range '{:?}' removed {} lines, new length {new_length}",
            self.range,
            line_count - buffer.line_count()
        );
    }
}

impl Display for LineDrop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LineDrop '{}:{:?}'", self.range.start, self.range.end)
    }
}

#[cfg(test)]
mod tests {
    use crate::ceos::buffer::buffer::Buffer;
    use crate::ceos::command::Command;
    use crate::ceos::command::filter::linedrop::LineDrop;

    const CONTENT: &str = "1 delete me\n\
        2 keep me\n\
        \n\
        3 delete me\n\
        4 keep me\n";

    #[test]
    fn test_filter_prefix() -> Result<(), ()> {
        let (sender, _) = std::sync::mpsc::channel();
        let mut buffer = Buffer::new_from_string(sender, CONTENT);
        assert_eq!(CONTENT.len(), buffer.len());
        assert_eq!(5, buffer.line_count());
        let filter = LineDrop::try_from("l ..2")?;
        filter.execute(&mut buffer);
        assert_eq!(3, buffer.line_count());
        assert_eq!("3 delete me", buffer.line_text(1));
        assert!(buffer.dirty);
        Ok(())
    }

    #[test]
    fn test_filter_range() -> Result<(), ()> {
        let (sender, _) = std::sync::mpsc::channel();
        let mut buffer = Buffer::new_from_string(sender, CONTENT);
        assert_eq!(CONTENT.len(), buffer.len());
        assert_eq!(5, buffer.line_count());
        let filter = LineDrop::try_from("l 3..")?;
        filter.execute(&mut buffer);
        assert_eq!(3, buffer.line_count());
        assert_eq!("2 keep me", buffer.line_text(1));
        assert!(buffer.dirty);
        Ok(())
    }

    #[test]
    fn test_filter_suffix() -> Result<(), ()> {
        let (sender, _) = std::sync::mpsc::channel();
        let mut buffer = Buffer::new_from_string(sender, CONTENT);
        assert_eq!(CONTENT.len(), buffer.len());
        assert_eq!(5, buffer.line_count());
        let filter = LineDrop::try_from("l 2..4")?;
        filter.execute(&mut buffer);
        assert_eq!(3, buffer.line_count());
        assert_eq!("2 keep me", buffer.line_text(1));
        assert!(buffer.dirty);
        Ok(())
    }
}
