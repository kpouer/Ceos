use std::cmp;
use std::fmt::Display;

use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Stroke, StrokeKind};
use egui::Ui;
use log::debug;

use crate::ceos::buffer::Buffer;
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
        textarea: &TextAreaProperties,
        _: usize,
        drawing_pos: Pos2,
    ) {
        let char_width = tools::char_width(textarea.font_id.clone(), ui);
        let end_x = if self.range.end.is_some() {
            self.range.end.unwrap() as f32 * char_width
        } else {
            ui.max_rect().width()
        };
        let top_left = Pos2::new(
            drawing_pos.x + self.range.start as f32 * char_width,
            drawing_pos.y,
        );
        let bottom_right = Pos2::new(drawing_pos.x + end_x, drawing_pos.y + textarea.line_height);
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
            "Applied filter removed {} lines, new length {new_length}",
            line_count - buffer.line_count()
        );
    }
}

impl ColumnFilter {
    pub(crate) fn apply_to_line(&self, line: &mut Line) {
        if self.range.start >= line.content.len() {
            return;
        }

        if let Some(end) = self.range.end {
            line.content
                .drain(self.range.start..cmp::min(line.content.len(), end));
        } else {
            line.content.drain(self.range.start..);
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
    ) -> anyhow::Result<(), ()> {
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
    fn test_filter_line_prefix() -> anyhow::Result<(), ()> {
        let filter = ColumnFilter::try_from("..2")?;
        let mut line = Line::from("1 delete me");
        filter.apply_to_line(&mut line);
        assert_eq!("delete me", line.content);
        Ok(())
    }

    #[test]
    fn test_filter_line_prefix_short() -> anyhow::Result<(), ()> {
        let filter = ColumnFilter::try_from("..2")?;
        let mut line = Line::from("1");
        filter.apply_to_line(&mut line);
        assert!(line.content.is_empty());
        Ok(())
    }

    #[test]
    fn test_filter_line_prefix_empty() -> anyhow::Result<(), ()> {
        let filter = ColumnFilter::try_from("..2")?;
        let mut line = Line::from("");
        filter.apply_to_line(&mut line);
        assert!(line.content.is_empty());
        Ok(())
    }

    #[test]
    fn test_filter() -> anyhow::Result<(), ()> {
        let filter = ColumnFilter::try_from("..2")?;
        let content = "1 delete me\n\
        2 keep me\n\
        \n\
        3 delete me\n\
        4 keep me\n";
        let mut buffer = Buffer::from(content);
        assert_eq!(content.len(), buffer.len());
        assert_eq!(5, buffer.line_count());
        filter.execute(&mut buffer);
        assert_eq!(content.len() - 8, buffer.len());
        Ok(())
    }
}
