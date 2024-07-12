use std::fmt::Display;

use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Color32, Stroke};
use egui::Ui;
use log::info;

use crate::ceos::command::Command;
use crate::textarea::buffer::Buffer;
use crate::textarea::buffer::line::Line;
use crate::textarea::buffer::line_status::LineStatus;
use crate::textarea::renderer::Renderer;
use crate::textarea::textareaproperties::TextAreaProperties;

pub(crate) struct Filter {
    command: String,
}

impl Filter {
    pub(crate) fn accept(&self, line: &Line) -> bool {
        line.content().contains(&self.command)
    }
}

impl TryFrom<&str> for Filter {
    type Error = String;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if command.starts_with("filter ") && command.len() > 7 {
            Ok(Self { command: command[7..].to_string() })
        } else {
            Err("Command not valid".to_string())
        }
    }
}

impl Renderer for Filter {
    fn paint_line(&self, ui: &mut Ui, textarea: &TextAreaProperties, line: usize, _: Pos2, drawing_pos: Pos2) {
        let line = &textarea
            .buffer()
            .content()[line];
        if self.accept(line) {
            let bottom_right = Pos2::new(ui.max_rect().max.x, drawing_pos.y + textarea.line_height());
            let line_rect = Rect::from_min_max(drawing_pos, bottom_right);
            let painter = ui.painter();
            painter.rect(line_rect, 0.0, Color32::RED, Stroke::default());
        }
    }
}

impl Command for Filter {
    fn execute(&self, buffer: &mut Buffer) {
        let line_count = buffer.content().len();
        buffer.content_mut().retain(|line| self.accept(line));

        buffer.content_mut()
            .iter_mut()
            .for_each(|line| line.set_status(LineStatus::Normal));
        let new_length = buffer.compute_total_length();
        info!("Applied filter '{}' removed {} lines, new length {new_length}",
            self.command, line_count - buffer.content().len());
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Filter '{}'", self.command)
    }
}
