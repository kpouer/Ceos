use std::fmt::Display;

use crate::ceos::command::Command;
use crate::ceos::gui::tools;
use crate::textarea::buffer::line::Line;
use crate::textarea::buffer::Buffer;
use crate::textarea::renderer::Renderer;
use crate::textarea::textareaproperties::TextAreaProperties;
use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Color32, Stroke};
use egui::Ui;
use log::info;

#[derive(Debug, PartialEq)]
pub(crate) struct ColumnFilter {
    start: usize,
    end: Option<usize>,
}

const SEPARATOR: &str = "..";

impl TryFrom<&str> for ColumnFilter {
    type Error = String;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if let Some(stripped) = command.strip_prefix(SEPARATOR) {
            let end = stripped.parse::<usize>().ok();
            if end.is_some() {
                return Ok(ColumnFilter { start: 0, end });
            }
        } else if let Some(stripped) = command.strip_suffix(SEPARATOR) {
            if let Ok(start) = stripped.parse::<usize>() {
                return Ok(ColumnFilter { start, end: None });
            }
        } else {
            let tokens: Vec<&str> = command.split(SEPARATOR).collect();
            if tokens.len() == 2 {
                if let Ok(start) = tokens.first().unwrap().parse::<usize>() {
                    let end = tokens.get(1).unwrap().parse::<usize>().ok();
                    if end.is_some() {
                        if start > end.unwrap() {
                            return Err("Invalid command".to_string());
                        }
                        return Ok(ColumnFilter { start, end });
                    }
                }
            }
        }
        Err("Invalid command".to_string())
    }
}

impl Renderer for ColumnFilter {
    fn paint_line(
        &self,
        ui: &mut Ui,
        textarea: &TextAreaProperties,
        _: usize,
        _: Pos2,
        drawing_pos: Pos2,
    ) {
        let char_width = tools::char_width(textarea.font_id().clone(), ui);
        let end_x = if self.end.is_some() {
            self.end.unwrap() as f32 * char_width
        } else {
            ui.max_rect().width()
        };
        let top_left = Pos2::new(
            drawing_pos.x + self.start as f32 * char_width,
            drawing_pos.y,
        );
        let bottom_right = Pos2::new(
            drawing_pos.x + end_x,
            drawing_pos.y + textarea.line_height(),
        );
        let line_rect = Rect::from_min_max(top_left, bottom_right);
        let painter = ui.painter();
        painter.rect(line_rect, 0.0, Color32::RED, Stroke::default());
    }
}

impl Command for ColumnFilter {
    fn execute(&self, buffer: &mut Buffer) {
        let line_count = buffer.content().len();

        buffer
            .content_mut()
            .iter_mut()
            .for_each(|line| self.apply_to_line(line));

        buffer
            .content_mut()
            .iter_mut()
            .for_each(|line| self.apply_to_line(line));
        let new_length = buffer.compute_total_length();
        info!(
            "Applied filter removed {} lines, new length {new_length}",
            line_count - buffer.content().len()
        );
    }
}

impl ColumnFilter {
    pub(crate) fn apply_to_line(&self, line: &mut Line) {
        let content = line.content_mut();
        if self.end.is_some() {
            content.drain(self.start..self.end.unwrap());
        } else {
            content.drain(self.start..);
        }
    }
}

impl Display for ColumnFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ColumnFilter '{}:{:?}'", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from() -> anyhow::Result<(), String> {
        let result = ColumnFilter::try_from("3:22")?;
        assert_eq!(
            ColumnFilter {
                start: 3,
                end: Some(22)
            },
            result
        );
        Ok(())
    }

    #[test]
    fn test_try_from_leading() -> anyhow::Result<(), String> {
        let result = ColumnFilter::try_from(":22")?;
        assert_eq!(
            ColumnFilter {
                start: 0,
                end: Some(22)
            },
            result
        );
        Ok(())
    }

    #[test]
    fn test_try_from_trailing() -> anyhow::Result<(), String> {
        let result = ColumnFilter::try_from("3:")?;
        assert_eq!(
            ColumnFilter {
                start: 3,
                end: None
            },
            result
        );
        Ok(())
    }

    #[test]
    fn test_try_from_invalid() -> anyhow::Result<(), String> {
        let result = ColumnFilter::try_from("33:22").is_err();
        assert!(result);
        Ok(())
    }
}
