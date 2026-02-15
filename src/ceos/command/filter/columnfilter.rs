use std::cmp;
use std::fmt::Display;

use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Stroke, StrokeKind};
use egui::Ui;
use log::debug;

use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::line::Line;
use crate::ceos::command::Command;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::gui::tools;
use crate::ceos::tools::range::Range;

#[derive(Debug, PartialEq)]
pub(crate) struct ColumnFilter {
    range: Range,
}

impl TryFrom<&str> for ColumnFilter {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(ColumnFilter {
            range: Range::try_from(value)?,
        })
    }
}

impl Renderer for ColumnFilter {
    fn paint_line(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        textarea_properties: &TextAreaProperties,
        _: usize,
        drawing_pos: Pos2,
        _has_focus: bool,
    ) {
        let char_width = tools::char_width(textarea_properties.font_id.clone(), ui);
        let end_x = match self.range.end {
            Some(end) => end as f32 * char_width,
            None => ui.max_rect().width(),
        };
        let top_left = Pos2::new(
            drawing_pos.x + self.range.start as f32 * char_width,
            drawing_pos.y,
        );
        let bottom_right = Pos2::new(drawing_pos.x + end_x, drawing_pos.y + textarea_properties.line_height);
        let line_rect = Rect::from_min_max(top_left, bottom_right);
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

impl Command for ColumnFilter {
    fn execute(&self, buffer: &mut Buffer) {
        let line_count = buffer.line_count();
        let new_length = buffer.filter_line_mut(|line| self.apply_to_line(line));
        debug!(
            "Applied filter removed {} lines, new length {}",
            line_count - buffer.line_count(),
            new_length
        );
    }
}

impl ColumnFilter {
    pub(crate) fn apply_to_line(&self, line: &mut Line) {
        if self.range.start >= line.len() {
            return;
        }

        if let Some(end) = self.range.end {
            line.drain(self.range.start..cmp::min(line.len(), end));
        } else {
            line.drain(self.range.start..);
        }
    }
}

impl Display for ColumnFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ColumnFilter '{}:{:?}'",
            self.range.start, self.range.end
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(3, Some(22), "3..22")]
    fn test_try_from(
        #[case] start: usize,
        #[case] end: Option<usize>,
        #[case] command: &str,
    ) -> Result<(), ()> {
        let result = ColumnFilter::try_from(command)?;
        assert_eq!(
            ColumnFilter {
                range: Range { start, end }
            },
            result
        );
        Ok(())
    }

    #[test]
    fn test_filter_line_prefix() -> Result<(), ()> {
        let filter = ColumnFilter::try_from("..2")?;
        let mut line = Line::from("1 delete me");
        filter.apply_to_line(&mut line);
        assert_eq!("delete me", line.content());
        Ok(())
    }

    #[test]
    fn test_filter_line_prefix_short() -> Result<(), ()> {
        let filter = ColumnFilter::try_from("..2")?;
        let mut line = Line::from("1");
        filter.apply_to_line(&mut line);
        assert!(line.is_empty());
        Ok(())
    }

    #[test]
    fn test_filter_line_prefix_empty() -> Result<(), ()> {
        let filter = ColumnFilter::try_from("..2")?;
        let mut line = Line::from("");
        filter.apply_to_line(&mut line);
        assert!(line.is_empty());
        Ok(())
    }

    #[test]
    fn test_filter() -> Result<(), ()> {
        let filter = ColumnFilter::try_from("..2")?;
        let content = "1 delete me\n\
        2 keep me\n\
        \n\
        3 delete me\n\
        4 keep me\n";
        let (sender, _) = std::sync::mpsc::channel();
        let mut buffer = Buffer::new_from_string(sender, content, 2);
        assert_eq!(content.len(), buffer.len());
        assert_eq!(5, buffer.line_count());
        filter.execute(&mut buffer);
        assert_eq!(content.len() - 8, buffer.len());
        Ok(())
    }
}
