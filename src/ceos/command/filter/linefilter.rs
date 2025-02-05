use crate::ceos::buffer::line::Line;
use crate::ceos::buffer::Buffer;
use crate::ceos::command::Command;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::event::Event;
use crate::event::Event::BufferLoaded;
use eframe::emath::{Pos2, Rect};
use eframe::epaint::Stroke;
use egui::{StrokeKind, Ui};
use log::info;
use std::fmt::Display;
use std::sync::mpsc::Sender;
use std::time::Instant;

pub(crate) struct LineFilter {
    sender: Sender<Event>,
    filters: Vec<String>,
}

impl LineFilter {
    pub(crate) fn accept(&self, line: &Line) -> bool {
        for filter in &self.filters {
            if let Some(prefix) = filter.strip_prefix('!') {
                // the search is !xxxxx so the line must not contains xxxxx or eventually
                // contains !xxxxx exacty
                if tools::contains(&line.content, prefix) && !tools::contains(&line.content, filter)
                {
                    return false;
                }
            } else if !tools::contains(&line.content, filter) {
                return false;
            }
        }
        true
    }
}

impl TryFrom<(&str, Sender<Event>)> for LineFilter {
    type Error = String;

    fn try_from((command, sender): (&str, Sender<Event>)) -> Result<Self, Self::Error> {
        if command.starts_with("filter ") && command.len() > 7 {
            let command = command[7..].split('&').map(|tok| tok.to_string()).collect();
            Ok(Self {
                sender,
                filters: command,
            })
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
        drawing_pos: Pos2,
    ) {
        let line = &textarea.buffer.content[line];
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
    fn execute(&self, mut buffer: Buffer) {
        let start = Instant::now();
        let line_count = buffer.line_count();
        let new_length = buffer.retain_line_mut(|line| self.accept(line));
        info!(
            "Applied filter '{:?}' removed {} lines, new length {new_length} in {} ms",
            self.filters,
            line_count - buffer.line_count(),
            start.elapsed().as_millis()
        );
        self.sender.send(BufferLoaded(buffer)).unwrap();
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
    fn test_filter() -> anyhow::Result<(), String> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let filter = LineFilter::try_from(("filter delete", sender))?;
        let content = "1 delete me\n\
        2 keep me\n\
        3 delete me\n\
        4 keep me\n";
        let buffer = Buffer::from(content);
        assert_eq!(content.len(), buffer.len());
        assert_eq!(4, buffer.line_count());
        filter.execute(buffer);
        if let BufferLoaded(buffer) = receiver.recv().unwrap() {
            assert!(buffer.dirty);
            assert_eq!(2, buffer.line_count());
            return Ok(());
        }
        Err("Missing buffer".to_string())
    }
}
