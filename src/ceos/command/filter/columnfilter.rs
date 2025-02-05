use crate::ceos::buffer::line::Line;
use crate::ceos::buffer::Buffer;
use crate::ceos::command::Command;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::gui::ui_tools;
use crate::event::Event;
use crate::event::Event::{BufferLoaded, TaskEnded, TaskStarted, TaskUpdated};
use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Stroke, StrokeKind};
use egui::Ui;
use log::{debug, info};
use std::fmt::Display;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{cmp, thread};
use tools::range::Range;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct ColumnFilter {
    sender: Sender<Event>,
    range: Range,
}

impl TryFrom<(&str, Sender<Event>)> for ColumnFilter {
    type Error = ();

    fn try_from((value, sender): (&str, Sender<Event>)) -> Result<Self, Self::Error> {
        let range = Range::try_from(value)?;
        Ok(ColumnFilter { sender, range })
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
        let char_width = ui_tools::char_width(textarea.font_id.clone(), ui);
        let end_x = if let Some(end) = self.range.end() {
            end as f32 * char_width
        } else {
            ui.max_rect().width()
        };
        let top_left = Pos2::new(
            drawing_pos.x + self.range.start() as f32 * char_width,
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
    fn execute(&self, mut buffer: Buffer) {
        let line_count = buffer.line_count();
        let refresh_rate = line_count / 100;
        let task_id = Uuid::new_v4().to_string();
        self.sender
            .send(TaskStarted(
                task_id.clone(),
                "ColumnFilter".to_string(),
                line_count,
            ))
            .unwrap();
        let sender = self.sender.clone();
        let range = self.range.clone();
        thread::spawn(move || {
            let start = Instant::now();
            let progress = AtomicUsize::new(0);
            let new_length = buffer.filter_line_mut(|line| {
                let index = progress.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if index % refresh_rate == 0 {
                    sender.send(TaskUpdated(task_id.clone(), index)).unwrap();
                    #[cfg(debug_assertions)]
                    thread::sleep(Duration::from_millis(20));
                }
                Self::apply_to_line(&range, line)
            });
            info!("Processed in {} ms", start.elapsed().as_millis());
            sender.send(TaskEnded(task_id)).unwrap();
            debug!(
                "Applied filter removed {} lines, new length {new_length}",
                line_count - buffer.line_count()
            );
            sender.send(BufferLoaded(buffer)).unwrap();
        });
    }
}

impl ColumnFilter {
    pub(crate) fn apply_to_line(range: &Range, line: &mut Line) {
        if range.start() >= line.content.len() {
            return;
        }

        if let Some(end) = range.end() {
            line.content
                .drain(range.start()..cmp::min(line.content.len(), end));
        } else {
            line.content.drain(range.start()..);
        }
    }
}

impl Display for ColumnFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ColumnFilter '{}:{:?}'",
            self.range.start(),
            self.range.end()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::sync::mpsc::channel;

    impl PartialEq for ColumnFilter {
        fn eq(&self, other: &Self) -> bool {
            self.range == other.range
        }
    }

    #[rstest]
    #[case(3, 22, "3..22")]
    fn test_try_from(
        #[case] start: usize,
        #[case] end: usize,
        #[case] command: &str,
    ) -> anyhow::Result<(), ()> {
        let (sender, _) = channel::<Event>();
        let result = ColumnFilter::try_from((command, sender.clone()))?;
        assert_eq!(
            ColumnFilter {
                sender,
                range: Range::new(start, end)?
            },
            result
        );
        Ok(())
    }

    #[test]
    fn test_filter_line_prefix() -> anyhow::Result<(), ()> {
        let (sender, _) = channel::<Event>();
        let filter = ColumnFilter::try_from(("..2", sender.clone()))?;
        let mut line = Line::from("1 delete me");
        ColumnFilter::apply_to_line(&filter.range, &mut line);
        assert_eq!("delete me", line.content);
        Ok(())
    }

    #[test]
    fn test_filter_line_prefix_short() -> anyhow::Result<(), ()> {
        let (sender, _) = channel::<Event>();
        let filter = ColumnFilter::try_from(("..2", sender.clone()))?;
        let mut line = Line::from("1");
        ColumnFilter::apply_to_line(&filter.range, &mut line);
        assert!(line.content.is_empty());
        Ok(())
    }

    #[test]
    fn test_filter_line_prefix_empty() -> anyhow::Result<(), ()> {
        let (sender, _) = channel::<Event>();
        let filter = ColumnFilter::try_from(("..2", sender.clone()))?;
        let mut line = Line::from("");
        ColumnFilter::apply_to_line(&filter.range, &mut line);
        assert!(line.content.is_empty());
        Ok(())
    }

    #[test]
    fn test_filter() -> anyhow::Result<(), ()> {
        let (sender, receiver) = channel::<Event>();
        let filter = ColumnFilter::try_from(("..2", sender.clone()))?;
        let content = "1 delete me\n\
        2 keep me\n\
        \n\
        3 delete me\n\
        4 keep me\n";
        let buffer = Buffer::from(content);
        assert_eq!(content.len(), buffer.len());
        assert_eq!(5, buffer.line_count());
        filter.execute(buffer);
        receiver.recv().unwrap();
        if let BufferLoaded(buffer) = receiver.recv().unwrap() {
            assert_eq!(content.len() - 8, buffer.len());
            return Ok(());
        }
        Err(())
    }
}
