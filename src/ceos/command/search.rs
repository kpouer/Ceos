use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::search::SearchMatcher;
use crate::ceos::search::simple_search_case_sensitive::SimpleSearchCaseSensitiveMatcher;
use crate::event::Event;
use crate::progress_operation::ProgressOperation;
use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Stroke, StrokeKind};
use egui::Ui;
use log::info;
use rayon::prelude::*;
use std::time::Instant;

/// Search filter
#[derive(Default, Debug)]
pub struct Search {
    search_matcher: Option<SimpleSearchCaseSensitiveMatcher>,
    // the line indexes containing the search value
    lines: Vec<usize>,
    index: usize,
}

impl TryFrom<&str> for Search {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        const PREFIX: &str = "s ";
        if command.starts_with(PREFIX) && command.len() > PREFIX.len() {
            let pattern = command[PREFIX.len()..].to_string();
            let search_matcher = SimpleSearchCaseSensitiveMatcher::new(&pattern, false);
            Ok(Self {
                search_matcher: Some(search_matcher),
                lines: Vec::new(),
                index: 0,
            })
        } else {
            Err(())
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
        _has_focus: bool,
    ) {
        let line = &textarea.buffer[line];
        if let Some(search_matcher) = &self.search_matcher
            && let Some((start, end)) = search_matcher.search(line.content(), 0)
        {
            let x1 = start as f32 * textarea.char_width;
            let x2 = end as f32 * textarea.char_width;
            let top_left = Pos2::new(drawing_pos.x + x1, drawing_pos.y);
            let bottom_right = Pos2::new(drawing_pos.x + x2, drawing_pos.y + textarea.line_height);
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
}

impl Search {
    pub fn init(&mut self, buffer: &Buffer) {
        let start = Instant::now();
        let _ = buffer.sender.send(Event::OperationStarted(
            ProgressOperation::Searching,
            buffer.line_groups().len(),
        ));
        let lines: Vec<usize> = if let Some(search_matcher) = &self.search_matcher {
            buffer
                .line_groups()
                .par_iter()
                .map(|line_group| (line_group.first_line(), line_group.lines()))
                .flat_map(|(first_line, lines)| {
                    let _ = buffer
                        .sender
                        .send(Event::OperationIncrement(ProgressOperation::Searching, 1));
                    lines
                        .iter()
                        .enumerate()
                        .filter_map(|(i, line)| {
                            search_matcher
                                .search(line.content(), 0)
                                .is_some()
                                .then_some(first_line + i)
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        } else {
            Vec::new()
        };
        self.lines = lines;
        let _ = buffer
            .sender
            .send(Event::OperationFinished(ProgressOperation::Searching));
        info!("Search took {}ms", start.elapsed().as_millis());
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    #[inline]
    pub(crate) const fn has_results(&self) -> bool {
        !self.lines.is_empty()
    }

    #[inline]
    pub(crate) const fn next(&mut self) {
        self.index = (self.index + 1) % self.lines.len();
    }

    pub(crate) const fn prev(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        if self.index == 0 {
            self.index = self.lines.len() - 1;
        } else {
            self.index -= 1;
        }
    }

    #[inline]
    pub(crate) fn line(&self) -> usize {
        self.lines[self.index]
    }

    #[inline]
    pub(crate) fn line_number(&self, index: usize) -> usize {
        self.lines[index]
    }

    #[inline]
    pub(crate) const fn result_count(&self) -> usize {
        self.lines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ceos::command::Action;
    use crate::ceos::command::filter::linefilter::LineFilter;

    #[test]
    fn test_filter() -> Result<(), ()> {
        let filter = LineFilter::try_from("filter delete")?;
        let content = "1 delete me\n\
        2 keep me\n\
        3 delete me\n\
        4 keep me\n";
        let (sender, _) = std::sync::mpsc::channel();
        let mut buffer = Buffer::new_from_string(sender, content, 2);
        assert_eq!(content.len(), buffer.len());
        assert_eq!(4, buffer.line_count());
        filter.execute(&mut buffer);
        assert_eq!(2, buffer.line_count());
        Ok(())
    }
}
