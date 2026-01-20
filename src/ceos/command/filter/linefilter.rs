use std::fmt::Display;
use std::time::Instant;
use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Stroke, StrokeKind};
use egui::Ui;
use log::info;

use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::line::Line;
use crate::ceos::command::Command;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;

#[derive(Debug)]
pub(crate) struct LineFilter {
    filters: Vec<String>,
}

impl LineFilter {
    pub(crate) fn accept(&self, line: &Line) -> bool {
        for filter in &self.filters {
            if let Some(prefix) = filter.strip_prefix('!') {
                if line.content().contains(prefix) && !line.content().contains(filter) {
                    return false;
                }
            } else if !line.content().contains(filter) {
                return false;
            }
        }
        true
    }
}

impl TryFrom<&str> for LineFilter {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        const PREFIX: &str = "filter ";

        command.strip_prefix(PREFIX)
            .filter(|rest| !rest.is_empty())
            .map(|rest| {
                let filters = rest.split('&')
                    .map(|tok| tok.to_string())
                    .collect();
                Self { filters }
            })
            .ok_or(())
    }
}

impl Renderer for LineFilter {
    fn paint_line(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        textarea: &TextAreaProperties,
        line: usize,
        drawing_pos: Pos2,
    ) {
        let line = &textarea.buffer[line];
        if !self.accept(line) {
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

impl Command for LineFilter {
    fn execute(&self, buffer: &mut Buffer) {
        let start = Instant::now();
        let line_count = buffer.line_count();
        let new_length = buffer.retain_line_mut(|line| self.accept(line));
        info!(
            "Applied filter '{:?}' removed {} lines, new length {new_length} in {}ms",
            self.filters,
            line_count - buffer.line_count(),
            start.elapsed().as_millis()
        );
    }
}

impl Display for LineFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Filter '{:?}'", self.filters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter() -> Result<(), ()> {
        let filter = LineFilter::try_from("filter delete")?;
        let content = "1 delete me\n\
        2 keep me\n\
        3 delete me\n\
        4 keep me\n";
        let (sender, _) = std::sync::mpsc::channel();
        let mut buffer = Buffer::new_from_string(sender, content);
        assert_eq!(content.len(), buffer.len());
        assert_eq!(4, buffer.line_count());
        filter.execute(&mut buffer);
        assert!(buffer.dirty);
        assert_eq!(2, buffer.line_count());
        Ok(())
    }
}
