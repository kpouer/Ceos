use std::fmt::Display;

use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Color32, Stroke};
use egui::Ui;
use log::info;

use crate::ceos::command::Command;
use crate::ceos::gui::theme;
use crate::ceos::gui::theme::Theme;
use crate::ceos::textarea::buffer::line::Line;
use crate::ceos::textarea::buffer::Buffer;
use crate::ceos::textarea::renderer::Renderer;
use crate::ceos::textarea::textareaproperties::TextAreaProperties;

pub(crate) struct LineFilter {
    command: Vec<String>,
}

impl LineFilter {
    pub(crate) fn accept(&self, line: &Line) -> bool {
        for command in &self.command {
            if let Some(suffix) = command.strip_prefix('!') {
                if line.content().contains(suffix) && !line.content().contains(command) {
                    return false;
                }
            } else {
                if !line.content().contains(command) {
                    return false;
                }
            }
        }
        true
    }
}

impl TryFrom<&str> for LineFilter {
    type Error = String;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if command.starts_with("filter ") && command.len() > 7 {
            let command = command[7..].split('&').map(|tok| tok.to_string()).collect();
            Ok(Self { command })
        } else {
            Err("Command not valid".to_string())
        }
    }
}

impl Renderer for LineFilter {
    fn paint_line(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        textarea: &TextAreaProperties,
        line: usize,
        _: Pos2,
        drawing_pos: Pos2,
    ) {
        let line = &textarea.buffer().content()[line];
        if self.accept(line) {
            let bottom_right =
                Pos2::new(ui.max_rect().max.x, drawing_pos.y + textarea.line_height());
            let line_rect = Rect::from_min_max(drawing_pos, bottom_right);
            let painter = ui.painter();
            painter.rect(line_rect, 0.0, theme.deleting, Stroke::default());
        }
    }
}

impl Command for LineFilter {
    fn execute(&self, buffer: &mut Buffer) {
        let line_count = buffer.content().len();
        buffer.content_mut().retain(|line| self.accept(line));

        let new_length = buffer.compute_length();
        info!(
            "Applied filter '{:?}' removed {} lines, new length {new_length}",
            self.command,
            line_count - buffer.content().len()
        );
    }
}

impl Display for LineFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Filter '{:?}'", self.command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter() -> anyhow::Result<(), String> {
        let filter = LineFilter::try_from("filter delete")?;
        let content = "1 delete me\n\
        2 keep me\n\
        3 delete me\n\
        4 keep me\n";
        let mut buffer = Buffer::new_from_text(content);
        assert_eq!(content.len(), buffer.len());
        assert_eq!(4, buffer.line_count());
        filter.execute(&mut buffer);
        assert_eq!(2, buffer.line_count());
        Ok(())
    }
}
