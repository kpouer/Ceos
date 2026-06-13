use crate::ceos::buffer::undo_manager::insert::Insert;
use crate::ceos::buffer::undo_manager::remove::Remove;
use crate::ceos::buffer::undo_manager::{UndoManager, UndoOperation};
use crate::event::Event;
use crate::progress_operation::ProgressOperation;
use buffer_core::caret_state::CaretState;
use buffer_core::line::Line;
use buffer_core::line_group::LineGroup;
use buffer_core::position::Position;
use buffer_core::text_range::Selection;
use buffer_core::text_range::TextRange;
use log::{debug, error, info, warn};
use rayon::prelude::*;
use std::borrow::Cow;
use std::ops::{Bound, Index, RangeBounds};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use tools::misc_tool::RangeTools;
use tools::text_tool::TextTool;

pub(crate) const DEFAULT_GROUP_SIZE: usize = 1000;
const LINE_SEPARATOR_LEN: usize = 1;

#[derive(Debug)]
pub struct Buffer {
    pub(crate) path: Option<PathBuf>,
    /// the linegroups, the last one is never full. Eventually it is empty
    content: Vec<LineGroup>,
    /// a decompressed group for temporary access
    tmp_decompressed_group: usize,
    /// The buffer length
    length: u64,
    /// The buffer line count
    line_count: usize,
    pub(crate) dirty: bool,
    pub(crate) sender: Sender<Event>,
    /// The size of the groups used for line compression.
    group_size: usize,
    undo_manager: UndoManager,
}

impl Buffer {
    #[cfg(debug_assertions)]
    pub(crate) fn lorem_ipsum(sender: Sender<Event>) -> Self {
        const LOREM_IPSUM: &str= "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi. Proin porttitor, orci nec nonummy molestie, enim est eleifend mi, non fermentum diam nisl sit amet erat. Duis semper. Duis arcu massa, scelerisque vitae, consequat in, pretium a, enim. Pellentesque congue. Ut in risus volutpat libero pharetra tempor. Cras vestibulum bibendum augue. Praesent egestas leo in pede. Praesent blandit odio eu enim. Pellentesque sed dui ut augue blandit sodales. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia Curae; Aliquam nibh. Mauris ac mauris sed pede pellentesque fermentum. Maecenas adipiscing ante non diam sodales hendrerit.
Ut velit mauris, egestas sed, gravida nec, ornare ut, mi. Aenean ut orci vel massa suscipit pulvinar. Nulla sollicitudin. Fusce varius, ligula non tempus aliquam, nunc turpis ullamcorper nibh, in tempus sapien eros vitae ligula. Pellentesque rhoncus nunc et augue. Integer id felis. Curabitur aliquet pellentesque diam. Integer quis metus vitae elit lobortis egestas. Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Morbi vel erat non mauris convallis vehicula. Nulla et sapien. Integer tortor tellus, aliquam faucibus, convallis id, congue eu, quam. Mauris ullamcorper felis vitae erat. Proin feugiat, augue non elementum posuere, metus purus iaculis lectus, et tristique ligula justo vitae magna.

Aliquam convallis sollicitudin purus. Praesent aliquam, enim at fermentum mollis, ligula massa adipiscing nisl, ac euismod nibh nisl eu lectus. Fusce vulputate sem at sapien. Vivamus leo. Aliquam euismod libero eu enim. Nulla nec felis sed leo placerat imperdiet. Aenean suscipit nulla in justo. Suspendisse cursus rutrum augue. Nulla tincidunt tincidunt mi. Curabitur iaculis, lorem vel rhoncus faucibus, felis magna fermentum augue, et ultricies lacus lorem varius purus. Curabitur eu amet.";
        Self::new_from_string(sender, LOREM_IPSUM, DEFAULT_GROUP_SIZE)
    }

    pub(crate) fn new_empty_buffer(sender: Sender<Event>) -> Self {
        let mut buffer = Self::new_with_group_size(sender, DEFAULT_GROUP_SIZE);
        buffer.push_line("");
        buffer
    }

    pub fn new_from_string(sender: Sender<Event>, text: &str, group_size: usize) -> Self {
        let mut buffer = Self::new_with_group_size(sender, group_size);

        let lines_iterator = text.lines();
        lines_iterator.into_iter().for_each(|line| {
            buffer.push_line(line);
        });

        buffer
    }

    #[inline]
    pub(super) fn new_with_group_size(sender: Sender<Event>, group_size: usize) -> Self {
        Self {
            path: None,
            content: vec![LineGroup::new(0, group_size)],
            tmp_decompressed_group: 0,
            length: 0,
            line_count: 0,
            dirty: false,
            sender,
            group_size,
            undo_manager: UndoManager::default(),
        }
    }

    pub(crate) fn normalize_selection(&self, selection: &mut Selection) -> bool {
        if selection.start.line >= self.line_count {
            return false;
        }
        let line_start_length = self.line_length(selection.start.line);
        selection.start.column = selection.start.column.min(line_start_length);

        selection.end.line = selection.end.line.min(self.line_count - 1);
        let line_end_length = if selection.is_single_line() {
            line_start_length
        } else {
            self.line_length(selection.end.line)
        };
        selection.end.column = selection.end.column.min(line_end_length);
        true
    }

    pub(crate) fn set_path(&mut self, path: PathBuf) {
        info!("set path to {path:?}");
        self.path = Some(path);
        self.dirty = true;
    }

    /// Compress all line groups and free their in-memory lines to reclaim memory.
    /// This is primarily intended for debug/maintenance actions.
    pub(crate) fn compress_all_groups(&mut self) {
        debug!("compress_all_groups");
        for line_group in &mut self.content {
            if line_group.is_empty() {
                // Nothing to compress in an empty group
                continue;
            }

            if line_group.is_decompressed() {
                line_group.eventually_compress();
            }
            // Free lines if present; debug_assert in free() ensures it's compressed
            line_group.free();
        }
    }

    /// Push a new line at the end of the buffer.
    /// It is called when creating a new buffer
    pub(super) fn push_line(&mut self, line: impl Into<Line>) {
        let line = line.into();
        let last_group = self.content.last_mut().expect("buffer is empty");

        self.length += (line.len() + LINE_SEPARATOR_LEN) as u64;
        self.line_count += 1;
        last_group.push(line);

        if last_group.is_full() {
            last_group.eventually_compress();
            last_group.free();

            let next_first = last_group.first_line() + last_group.line_count();
            self.content
                .push(LineGroup::new(next_first, self.group_size));
        }
    }

    /// Deletes content within a specified range of text.
    ///
    /// # Parameters
    /// - `text_range`: A `TextRange` struct specifying the range of text to be deleted.
    ///
    pub(crate) fn delete_range(&mut self, text_range: TextRange) {
        if self.line_count == 0
            || text_range.start.line >= self.line_count
            || text_range.is_empty()
            || text_range.end.line >= self.line_count
        {
            warn!("delete_range: invalid range {text_range:?}");
            return;
        }

        if text_range.start.line == text_range.end.line {
            self.delete_in_line(
                text_range.start.line,
                text_range.start.column..text_range.end.column,
            );
        } else {
            self.delete_in_lines(text_range);
        }

        self.compute_metadata();
        self.recompute_first_lines();
        self.dirty = true;
    }

    // Delete a range of text in a line
    fn delete_in_line<R>(&mut self, line_index: usize, range: R)
    where
        R: RangeBounds<usize> + Clone,
    {
        if let Some((group_index, line_in_group)) = self.find_group_index(line_index) {
            let line_group = &mut self.content[group_index];
            let remove = line_group.filter_line_mut(line_in_group, |line| {
                let start_col = RangeTools::start_bound(&range);
                let removed_text = line.drain(range.clone()).as_str().to_string();
                let position = Position {
                    line: line_index,
                    column: start_col,
                };
                Remove::new(
                    TextRange::new(
                        position,
                        Position {
                            line: line_index,
                            column: start_col + removed_text.len(),
                        },
                    ),
                    vec![removed_text],
                )
            });
            if let Some(remove) = remove {
                self.undo_manager
                    .push_undo(UndoOperation::Remove(remove), true);
            }
        } else {
            error!("delete_in_line: line index out of bounds {line_index}");
        }
    }

    /// Delete text on multiple lines
    fn delete_in_lines(&mut self, text_range: TextRange) {
        let Some((start_group_index, start_line_in_group)) =
            self.find_group_index(text_range.start.line)
        else {
            warn!("start_line out of bounds");
            return;
        };
        let end_line = text_range.end.line.min(self.line_count.saturating_sub(1));
        let Some((end_group_index, end_line_in_group)) = self.find_group_index(end_line) else {
            warn!("end_line out of bounds");
            return;
        };

        let suffix = {
            let end_group = &mut self.content[end_group_index];
            // the suffix is the remaining text of the last line of the range
            end_group.line(end_line_in_group)[text_range.end.column..].to_owned()
        };

        let first_group = &mut self.content[start_group_index];

        // let's process the first group first.
        // We will remove the end of the first line then add the suffix.
        let mut removed_content = Vec::with_capacity(text_range.line_count());
        first_group.filter_line_mut(start_line_in_group, |line| {
            let removed = line.drain(text_range.start.column..).as_str().to_owned();
            line.push_str(&suffix);
            removed_content.push(removed);
        });

        // now we have to drain the remaining lines
        if start_group_index == end_group_index {
            let drain = first_group.drain_lines(start_line_in_group + 1..=end_line_in_group);
            if let Some(drain_lines) = drain {
                removed_content.extend(drain_lines.into_iter().map(Line::into_content));
                if let Some(last) = removed_content.last_mut() {
                    last.drain(text_range.end.column..);
                }
            }
        } else {
            let drain = first_group.drain_lines(start_line_in_group + 1..);
            if let Some(drain_lines) = drain {
                drain_lines.into_iter().for_each(|line| {
                    removed_content.push(line.into_content());
                });
            }
            // drain the linegroups between the start and the end group
            if start_group_index + 1 < end_group_index {
                let drain = self.content.drain(start_group_index + 1..end_group_index);
                drain
                    .into_iter()
                    .flat_map(|line_group| line_group.into_iter())
                    .for_each(|line| removed_content.push(line.into_content()));
            }

            if start_group_index != end_group_index {
                let end_group = &mut self.content[end_group_index];
                // the last group is different we have to drain it's first lines too.
                let drain_lines = end_group.drain_lines(0..=end_line_in_group);
                if let Some(drain_lines) = drain_lines {
                    drain_lines.into_iter().for_each(|line| {
                        removed_content.push(line.into_content());
                    });
                }
            }
        };

        if removed_content.is_empty() {
            return;
        }
        let remove = Remove::new(text_range, removed_content);
        self.undo_manager
            .push_undo(UndoOperation::Remove(remove), true);
    }

    pub(crate) fn undo(&mut self) -> Option<CaretState> {
        if let Some(edit) = self.undo_manager.pop_undo() {
            self.undo_manager.start_operation();
            let new_position = edit.undo(self);
            self.undo_manager.end_operation();
            self.undo_manager.push_redo(edit);
            return Some(new_position);
        }
        None
    }

    pub(crate) fn redo(&mut self) -> Option<CaretState> {
        if let Some(edit) = self.undo_manager.pop_redo() {
            self.undo_manager.start_operation();
            let new_position = edit.redo(self);
            self.undo_manager.end_operation();
            self.undo_manager.push_undo(edit, false);
            return Some(new_position);
        }
        None
    }

    pub(crate) const fn can_undo(&self) -> bool {
        self.undo_manager.can_undo()
    }

    pub(crate) fn line_groups(&self) -> &[LineGroup] {
        &self.content
    }

    pub(crate) fn drain_line_mut<R>(&mut self, range: R) -> u64
    where
        R: RangeBounds<usize>,
    {
        // Convert RangeBounds to concrete start..end
        let (start_line, end_line) = self.normalize_range(range);
        if start_line >= end_line {
            let (new_length, _) = self.compute_metadata();
            return new_length;
        }

        let (start_group_index, start_line_in_group) = self
            .find_group_index(start_line)
            .expect("start_line out of bounds");
        let (end_group_index, end_line_in_group) = self
            .find_group_index(end_line - 1)
            .expect("end_line out of bounds");

        let start_group_line_count = self.content[start_group_index].line_count();
        let should_remove_first_group = start_line_in_group == 0
            && (start_group_index < end_group_index
                || end_line_in_group == start_group_line_count - 1);

        if !should_remove_first_group {
            let end_in_group = if start_group_index == end_group_index {
                end_line_in_group + 1
            } else {
                start_group_line_count
            };
            self.content[start_group_index].drain_lines(start_line_in_group..end_in_group);
        }

        if start_group_index != end_group_index {
            let first_group_to_remove = if should_remove_first_group {
                start_group_index
            } else {
                start_group_index + 1
            };
            let end_group = &mut self.content[end_group_index];
            if end_line_in_group == end_group.line_count() - 1 {
                // all the last group has to be removed
                self.content.drain(first_group_to_remove..=end_group_index);
            } else {
                end_group.drain_lines(..=end_line_in_group);
                self.content.drain(first_group_to_remove..end_group_index);
            }
        } else if should_remove_first_group {
            self.content.remove(start_group_index);
        }

        let (new_length, _) = self.compute_metadata();
        self.recompute_first_lines();
        self.dirty = true;
        new_length
    }

    pub(crate) fn filter_line_mut<R>(
        &mut self,
        line_number: usize,
        filter: impl FnMut(&mut Line) -> R,
    ) -> Option<R> {
        let Some((group_index, line_index)) = self.find_group_index(line_number) else {
            warn!("line_index out of bounds");
            return None;
        };
        let line_group = &mut self.content[group_index];
        line_group.filter_line_mut(line_index, filter)
    }

    pub(crate) fn filter_lines_mut<F>(&mut self, filter: F) -> u64
    where
        F: FnMut(&mut Line) + Clone + Sync,
    {
        let _ = self.sender.send(Event::OperationStarted(
            ProgressOperation::Filtering,
            self.content.len(),
        ));
        self.content.par_iter_mut().for_each(|line_group| {
            let _ = self
                .sender
                .send(Event::OperationIncrement(ProgressOperation::Filtering, 1));
            line_group.filter_lines_mut(filter.clone());
        });
        let (new_length, _) = self.compute_metadata();
        self.dirty = true;
        let _ = self
            .sender
            .send(Event::OperationFinished(ProgressOperation::Filtering));
        new_length
    }

    pub(crate) fn retain_line_mut<F>(&mut self, filter: F) -> u64
    where
        F: Fn(&Line) -> bool + Sync + Send + Clone,
    {
        let _ = self.sender.send(Event::OperationStarted(
            ProgressOperation::Filtering,
            self.content.len(),
        ));
        self.content.par_iter_mut().for_each(|line_group| {
            let _ = self
                .sender
                .send(Event::OperationIncrement(ProgressOperation::Filtering, 1));
            line_group.retain(filter.clone());
        });
        // remove empty groups
        self.content.retain(|g| !g.is_empty());
        let (new_length, _) = self.compute_metadata();
        self.recompute_first_lines();
        self.dirty = true;
        let _ = self
            .sender
            .send(Event::OperationFinished(ProgressOperation::Filtering));
        new_length
    }

    /// Decompress only the line groups that intersect with the provided line range.
    /// This is a preparatory pass to ensure subsequent read operations on that
    /// span won't trigger on-demand decompression.
    ///
    /// The range is expressed in line indices (0-based, end-exclusive when Excluded/Unbounded).
    pub(crate) fn prepare_range_for_read<R: RangeBounds<usize>>(&mut self, range: R) {
        use std::ops::Bound;

        let total_lines = self.line_count;

        // Normalize start
        let mut start = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s.saturating_add(1),
            Bound::Unbounded => 0,
        };

        // Normalize end
        let mut end = match range.end_bound() {
            Bound::Included(&e) => e.saturating_add(1),
            Bound::Excluded(&e) => e,
            Bound::Unbounded => total_lines,
        };

        // Clamp to valid bounds
        if start > total_lines {
            start = total_lines;
        }
        if end > total_lines {
            end = total_lines;
        }
        if start >= end {
            return;
        }

        // Define a window to keep around the requested range to avoid thrashing.
        // Groups entirely outside this window will be recompressed.
        let window_start = start.saturating_sub(self.group_size);
        let window_end = (end + self.group_size).min(total_lines);

        // Walk groups and decompress those intersecting [start, end),
        // recompress those fully outside [window_start, window_end).
        let mut acc: usize = 0; // cumulative line count before current group
        for g in &mut self.content {
            let g_lines = g.line_count();
            let g_start = acc;
            let g_end = acc + g_lines;

            // check interval intersection with the exact read range
            if g_end > start && g_start < end {
                g.eventually_decompress();
            } else if g_end <= window_start || g_start >= window_end {
                g.eventually_compress();
                g.free();
            }

            acc = g_end;
            // do not early-break; we may need to compress groups after the end
        }
    }

    pub(crate) fn line_length(&self, line: usize) -> usize {
        if let Some((gi, li)) = self.find_group_index(line) {
            return self.content[gi][li].len();
        }
        0
    }

    /// Returns the text of the line at the given index.
    /// The given index is 0-based
    pub(crate) fn line_text(&self, line: usize) -> Cow<'_, str> {
        let (gi, li) = self
            .find_group_index(line)
            .expect("line index out of bounds");
        self.content[gi].line(li)
    }

    pub(crate) fn get_text(&self, selection: &Selection) -> String {
        let first_line = self.line_text(selection.start.line);
        let first_line_text_tool = TextTool::new(&first_line);
        if selection.is_single_line() {
            return first_line_text_tool[selection.start.column..selection.end.column].to_string();
        }

        let mut result = first_line_text_tool[selection.start.column..].to_string();
        for line in selection.start.line + 1..selection.end.line {
            result.push('\n');
            result.push_str(&self.line_text(line));
        }
        result.push('\n');
        let last_line = self.line_text(selection.end.line);
        let last_line_text_tool = TextTool::new(&last_line);
        result.push_str(&last_line_text_tool[..selection.end.column]);

        result
    }

    /// Returns the text of the line at the given index.
    /// The given index is 0-based
    pub(crate) fn line_text_with_decompress(&mut self, line: usize) -> &str {
        let (gi, li) = self
            .find_group_index(line)
            .expect("line index out of bounds");

        if self.content[gi].is_compressed() {
            self.content[self.tmp_decompressed_group].eventually_compress();
            self.content[gi].eventually_decompress();
            self.tmp_decompressed_group = gi;
        }
        self.content[gi][li].content()
    }

    pub(crate) fn line_count(&self) -> usize {
        self.line_count
    }

    pub(crate) fn insert_char(&mut self, position: Position, ch: char) {
        if ch == '\n' {
            self.insert_newline(position);
            return;
        }
        if let Some((gi, li)) = self.find_group_index(position.line) {
            self.content[gi].filter_line_mut(li, |line| {
                line.insert(position.column, ch);
            });
            self.undo_manager.push_undo(
                UndoOperation::Insert(Insert::new(position, vec![ch.to_string()])),
                true,
            );

            self.compute_metadata();
            self.dirty = true;
        }
    }

    pub(crate) fn insert_newline(&mut self, position: Position) {
        if let Some((group_index, relative_line_index)) = self.find_group_index(position.line) {
            let line_group = &mut self.content[group_index];
            self.undo_manager.push_undo(
                UndoOperation::Insert(Insert::new(position, vec![String::new(), String::new()])),
                true,
            );
            let suffix = line_group.filter_line_mut(relative_line_index, |line| {
                line.drain(position.column..).as_str().to_owned()
            });
            if let Some(suffix) = suffix {
                line_group.insert_line(relative_line_index + 1, suffix);
            }
            self.compute_metadata();
            self.recompute_first_lines();
            self.dirty = true;
        }
    }

    /// Insert some text at the given position.
    /// The text must not be multiline
    pub(crate) fn insert_str(&mut self, position: Position, text: &str) {
        debug_assert!(!text.is_empty());
        debug_assert!(!text.contains("\n"));
        if let Some((gi, li)) = self.find_group_index(position.line) {
            let line_group = &mut self.content[gi];
            line_group.filter_line_mut(li, |line| {
                line.insert_str(position.column, text);
            });
            self.compute_metadata();
            self.dirty = true;
        }
    }

    pub(crate) fn insert_lines(&mut self, line_index: usize, lines: Vec<String>) {
        if lines.is_empty() {
            warn!("insert_lines called with empty lines");
            return;
        }
        if let Some((gi, li)) = self.find_group_index(line_index) {
            let line_group = &mut self.content[gi];
            for (i, line_text) in lines.into_iter().enumerate() {
                line_group.insert_line(li + i, line_text);
            }
            self.compute_metadata();
            self.recompute_first_lines();
            self.dirty = true;
        } else if line_index == self.line_count {
            for line_text in lines {
                self.push_line(line_text);
            }
        }
    }

    /// Returns the buffer length.
    /// It is the number of chars + end of lines
    #[inline]
    pub(crate) const fn len(&self) -> u64 {
        self.length
    }

    pub(crate) fn max_line_length(&self) -> usize {
        self.content
            .iter()
            .map(LineGroup::max_line_length)
            .max()
            .unwrap_or(0)
    }

    pub(crate) const fn group_count(&self) -> usize {
        self.content.len()
    }

    pub(crate) fn compressed_group_count(&self) -> usize {
        self.content.iter().filter(|g| g.is_compressed()).count()
    }

    pub(crate) fn decompressed_group_count(&self) -> usize {
        self.content.iter().filter(|g| g.is_decompressed()).count()
    }

    pub(crate) fn decompressed_line_count(&self) -> usize {
        self.content
            .iter()
            .map(LineGroup::decompressed_line_count)
            .sum()
    }

    fn compute_metadata(&mut self) -> (u64, usize) {
        let (length, line_count) = self
            .content
            .iter()
            .map(|line_group| (line_group.len(), line_group.line_count()))
            .reduce(|(l1, lc1), (l2, lc2)| (l1 + l2, lc1 + lc2))
            .unwrap_or((0, 0));
        self.length = length;
        self.line_count = line_count;
        (self.length, self.line_count)
    }

    pub(crate) fn mem(&self) -> usize {
        let vec_overhead = std::mem::size_of::<Vec<LineGroup>>();
        let array_mem = self.content.capacity() * std::mem::size_of::<LineGroup>();
        let groups_mem: usize = self.content.iter().map(LineGroup::mem).sum();
        vec_overhead + array_mem + groups_mem
    }

    pub(crate) fn compressed_size(&self) -> usize {
        self.content.iter().map(LineGroup::compressed_size).sum()
    }

    /// Each linegroup know it's first line.
    /// todo : optimize this
    fn recompute_first_lines(&mut self) {
        let mut first_line = 0;
        for g in &mut self.content {
            g.set_first_line(first_line);
            first_line += g.line_count();
        }
    }

    /// Finds the index of the group and the corresponding line within that group,
    /// given a line number in the aggregated content.
    ///
    /// # Arguments
    ///
    /// * `line` - The zero-based line number in the aggregated content to locate.
    ///
    /// # Returns
    ///
    /// * `Some((usize, usize))` - A tuple containing:
    ///   - The index of the group (`usize`).
    ///   - The corresponding line number relative to the group (`usize`).
    /// * `None` - If the given line number exceeds the total number of lines
    ///   in all groups combined.
    fn find_group_index(&self, mut line: usize) -> Option<(usize, usize)> {
        for (group_index, line_group) in self.content.iter().enumerate() {
            if line < line_group.line_count() {
                return Some((group_index, line));
            }
            line -= line_group.line_count();
        }
        None
    }

    fn normalize_range<R: RangeBounds<usize>>(&self, range: R) -> (usize, usize) {
        // todo : use let Range { start, end } = slice::range(range, ..len);
        let start = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&e) => e + 1,
            Bound::Excluded(&e) => e,
            Bound::Unbounded => self.line_count,
        };
        (start.min(self.line_count), end.min(self.line_count))
    }

    #[cfg(test)]
    fn debug(&self) {
        println!("Buffer Debug Info:");
        println!("Line Count: {}", self.line_count);
        println!("Dirty: {}", self.dirty);
        println!("Content:");
        for line_group in &self.content {
            line_group.debug();
        }
    }

    #[cfg(test)]
    pub(crate) fn new_empty_test_buffer() -> Buffer {
        let (sender, _) = std::sync::mpsc::channel();
        Buffer::new_with_group_size(sender, 2)
    }

    #[cfg(test)]
    pub(crate) fn new_test_buffer(str: &str, group_size: usize) -> Buffer {
        let (sender, _) = std::sync::mpsc::channel();
        Buffer::new_from_string(sender, str, group_size)
    }
}

#[cfg(test)]
impl std::fmt::Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self
            .content
            .iter()
            .map(|line_group| {
                line_group.decompress_lines();
                line_group.to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{str}")
    }
}

impl Index<usize> for Buffer {
    type Output = Line;

    fn index(&self, index: usize) -> &Self::Output {
        let (group_index, line_index) = self
            .find_group_index(index)
            .expect("line index out of bounds");
        &self.content[group_index][line_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffer_core::position::Position;

    #[test]
    fn from_str_builds_lines_and_lengths() {
        let mut b = Buffer::new_test_buffer("a\nbb\nccc", 2);
        assert_eq!(b.line_count, 3);
        // Each line counted as len+1 in our model
        assert_eq!(b.len(), (1 + 1) + (2 + 1) + (3 + 1));
        assert_eq!(b.max_line_length(), 3);
        b.prepare_range_for_read(..);
        assert_eq!(b.line_text(0), "a");
        assert_eq!(b[1].content(), "bb");
    }

    #[test]
    fn iter_yields_all_lines_in_order() {
        let b = Buffer::new_test_buffer("l1\nl2\nl3", 2);
        let mut collected = Vec::new();
        b.line_groups().iter().for_each(|line_group| {
            line_group
                .lines()
                .iter()
                .map(|l| l.content().to_string())
                .for_each(|l| collected.push(l))
        });
        assert_eq!(collected, vec!["l1", "l2", "l3"]);
    }

    #[test]
    fn group_boundary_and_compression_path() {
        // Push exactly DEFAULT_GROUP_SIZE lines to trigger compression of first group
        let mut b = Buffer::new_empty_test_buffer();
        for i in 0..b.group_size {
            b.push_line(format!("{:03}", i));
        }
        // We should still report correct counts and access
        assert_eq!(b.line_count, b.group_size);
        assert_eq!(b.max_line_length(), 3);
        // Access a few positions
        b.prepare_range_for_read(0..10);
        assert_eq!(b.line_text(0), "000");
        let start = if b.group_size > 10 {
            b.group_size - 10
        } else {
            0
        };
        b.prepare_range_for_read(start..b.group_size + 100);
        assert_eq!(
            b.line_text(b.group_size - 1),
            format!("{:03}", b.group_size - 1)
        );
    }

    #[test]
    fn filter_line_mut_updates_all_lines() {
        let mut b = Buffer::new_test_buffer("a\nbb", 2);
        let new_len = b.filter_lines_mut(|line| {
            let mut s = line.to_string();
            s.push('x');
            *line = Line::from(s);
        });
        assert!(new_len >= b.len());
        b.prepare_range_for_read(..);
        assert!(b.line_text(0).ends_with('x'));
        assert!(b.line_text(1).ends_with('x'));
        assert!(b.dirty);
    }

    #[test]
    fn retain_line_mut_keeps_predicate_matches() {
        let mut b = Buffer::new_test_buffer("a\nbb\nccc\ndddd", 2);
        let _ = b.retain_line_mut(|l| l.len() % 2 == 0); // keep even lengths: 2 and 4
        assert_eq!(b.line_count, 2);
        assert_eq!(b.line_text(0), "bb");
        assert_eq!(b.line_text(1), "dddd");
        assert!(b.dirty);
    }

    #[test]
    fn drain_line_mut_various_ranges() {
        let mut buffer = Buffer::new_test_buffer("l0\nl1\nl2\nl3\nl4", 2);
        // buffer has 5 lines: ["l0", "l1", "l2", "l3", "l4"]
        // group_size = 2
        // groups: G0: [l0, l1], G1: [l2, l3], G2: [l4]

        let _ = buffer.drain_line_mut(1..3); // remove l1, l2
        // expected: ["l0", "l3", "l4"]
        // groups: G0: [l0], G1: [l3], G2: [l4] (or merged, but drain_line_mut doesn't merge)
        buffer.debug();
        assert_eq!(buffer.line_count, 3);
        buffer.prepare_range_for_read(..);
        assert_eq!(buffer.line_text(0), "l0");
        assert_eq!(buffer.line_text(1), "l3");
        assert_eq!(buffer.line_text(2), "l4");

        // Remove last element with inclusive range
        let mut buffer = Buffer::new_test_buffer("l0\nl1\nl2", 2);
        let _ = buffer.drain_line_mut(2..=2);
        assert_eq!(buffer.line_count, 2);
        buffer.prepare_range_for_read(..);
        assert_eq!(buffer.line_text(0), "l0");
        assert_eq!(buffer.line_text(1), "l1");

        // Remove all lines
        let mut buffer = Buffer::new_test_buffer("l0\nl1\nl2", 2);
        let _ = buffer.drain_line_mut(..);
        assert_eq!(buffer.line_count, 0);
    }

    #[test]
    fn drain_line_mut_bug_reproduction() {
        // Test 1: Drain spanning multiple groups
        let mut buffer = Buffer::new_test_buffer("l0\nl1\nl2\nl3\nl4", 2);
        // Groups: G0:[l0, l1], G1:[l2, l3], G2:[l4]
        // Drain 1..4 (l1, l2, l3)
        buffer.drain_line_mut(1..4);
        // Expected: ["l0", "l4"]
        assert_eq!(
            buffer.line_count, 2,
            "Line count should be 2 after draining 1..4"
        );
        buffer.prepare_range_for_read(..);
        assert_eq!(buffer.line_text(0), "l0");
        assert_eq!(buffer.line_text(1), "l4");

        // Test 2: should_remove_first_group when spanning multiple groups
        let mut buffer = Buffer::new_test_buffer("l0\nl1\nl2\nl3", 2);
        // Groups: G0:[l0, l1], G1:[l2, l3]
        // Drain 0..3 (l0, l1, l2)
        // start_line_in_group = 0
        // lines_to_delete = 3
        // line_group.line_count = 2
        // current code's should_remove_first_group = (0 == 0 && 2 == 3) => false
        // BUT it SHOULD remove G0 because l0, l1 are both being deleted.
        buffer.drain_line_mut(0..3);
        assert_eq!(buffer.line_count, 1);
        buffer.prepare_range_for_read(..);
        assert_eq!(buffer.line_text(0), "l3");
    }

    #[test]
    fn prepare_range_for_read_safe_and_accessible() {
        let mut b = Buffer::new_empty_test_buffer();
        for i in 0..(DEFAULT_GROUP_SIZE * 2 + 10) {
            b.push_line(format!("line{}", i));
        }
        // Should not panic and should allow access to middle range
        b.prepare_range_for_read(DEFAULT_GROUP_SIZE - 5..DEFAULT_GROUP_SIZE + 5);
        assert_eq!(
            b.line_text(DEFAULT_GROUP_SIZE),
            format!("line{}", DEFAULT_GROUP_SIZE)
        );
    }

    #[test]
    fn delete_range_single_line() {
        let mut b = Buffer::new_test_buffer("abcdef", 2);
        b.delete_range(TextRange::new(Position::new(0, 2), Position::new(0, 5)));
        assert_eq!(b.line_text(0), "abf");
        assert_eq!(b.line_count, 1);
        assert!(b.dirty);
    }

    #[test]
    fn delete_range_multi_line_merges() {
        let mut b = Buffer::new_test_buffer("hello\nworld\n!!!", 2);
        b.delete_range(TextRange::new(Position::new(0, 2), Position::new(1, 3)));
        assert_eq!(b.line_text(0), "held");
        assert_eq!(b.line_text(1), "!!!");
        assert_eq!(b.line_count, 2);
        assert!(b.dirty);
    }

    #[test]
    fn delete_range_to_line_start() {
        const TEXT: &str = "aaa\n\
        bbb\n\
        ccc";
        let mut buffer = Buffer::new_test_buffer(TEXT, 2);
        let range = TextRange::new(Position::new(0, 1), Position::new(2, 1));
        buffer.delete_range(range);
        buffer.debug();
        assert_eq!(buffer.line_text(0), "acc");
        assert_eq!(buffer.line_count, 1);
        assert!(buffer.dirty);
    }

    #[test]
    fn mem_non_decreasing_after_growth() {
        let mut b = Buffer::new_empty_test_buffer();
        let base = b.mem();
        b.push_line("abc");
        assert!(b.mem() >= base);
    }

    #[test]
    fn test_undo_remove_range() {
        let input = "Hello World";
        let mut buffer = Buffer::new_test_buffer(input, 100);
        buffer.delete_range(TextRange::new(Position::new(0, 5), Position::new(0, 11)));
        assert_eq!(buffer.line_text(0), "Hello");
        assert!(buffer.undo_manager.can_undo());
        if let Some(edit) = buffer.undo_manager.last_undo() {
            assert_eq!(
                "Remove { text_range: TextRange { start: Position { line: 0, column: 5 }, end: Position { line: 0, column: 11 } }, lines: [\" World\"] }",
                edit.to_string()
            );
        }
        buffer.undo();
        assert_eq!(buffer.line_text(0), input);
    }

    #[test]
    fn test_undo_delete_across_lines() {
        let input = "AAAA 1\nBBBB 2\nCCCC 3";
        let mut buffer = Buffer::new_test_buffer(input, 100);
        buffer.delete_range(TextRange::new(Position::new(0, 4), Position::new(2, 4)));
        assert_eq!(buffer.line_count, 1);
        assert_eq!(buffer.line_text(0), "AAAA 3");
        assert!(buffer.undo_manager.can_undo());
        if let Some(edit) = buffer.undo_manager.last_undo() {
            assert_eq!(
                "Remove { text_range: TextRange { start: Position { line: 0, column: 4 }, end: Position { line: 2, column: 4 } }, lines: [\" 1\", \"BBBB 2\", \"CCCC\"] }",
                edit.to_string()
            );
        }
        buffer.undo();
        assert_eq!(buffer.line_count, 3);
        let content = buffer.to_string();
        assert_eq!(input, content);
    }

    #[test]
    fn test_insert_char_at_line_beginning() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        buffer.insert_char(Position::ZERO, 'X');
        assert_eq!(buffer.line_text(0), "Xhello");
        assert!(buffer.dirty);
    }

    #[test]
    fn test_insert_char_in_line_middle() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        buffer.insert_char(Position::new(0, 2), 'X');
        assert_eq!(buffer.line_text(0), "heXllo");
        assert!(buffer.dirty);
    }

    #[test]
    fn test_insert_char_at_line_end() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        buffer.insert_char(Position::new(0, 5), 'X');
        assert_eq!(buffer.line_text(0), "helloX");
        assert!(buffer.dirty);
    }

    #[test]
    fn test_insert_newline_splits_line() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        let initial_length = buffer.len();
        buffer.insert_newline(Position::new(0, 2));
        assert_eq!(buffer.line_count, 2);
        assert_eq!(buffer.line_text(0), "he");
        assert_eq!(buffer.line_text(1), "llo");
        assert!(buffer.len() > initial_length);
        assert!(buffer.dirty);
    }

    #[test]
    fn test_insert_newline_at_line_beginning() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        buffer.insert_newline(Position::ZERO);
        assert_eq!(buffer.line_count, 2);
        assert_eq!(buffer.line_text(0), "");
        assert_eq!(buffer.line_text(1), "hello");
        assert!(buffer.dirty);
    }

    #[test]
    fn test_insert_newline_at_line_end() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        buffer.insert_newline(Position::new(0, 5));
        assert_eq!(buffer.line_count, 2);
        assert_eq!(buffer.line_text(0), "hello");
        assert_eq!(buffer.line_text(1), "");
        assert!(buffer.dirty);
    }

    #[test]
    fn test_insert_char_newline_using_insert_char() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        buffer.insert_char(Position::new(0, 2), '\n');
        assert_eq!(buffer.line_count, 2);
        assert_eq!(buffer.line_text(0), "he");
        assert_eq!(buffer.line_text(1), "llo");
    }

    #[test]
    fn test_insert_multiple_chars_sequentially() {
        let mut buffer = Buffer::new_test_buffer("ab", 2);
        buffer.insert_char(Position::new(0, 1), 'X');
        buffer.insert_char(Position::new(0, 2), 'Y');
        buffer.insert_char(Position::new(0, 3), 'Z');
        assert_eq!(buffer.line_text(0), "aXYZb");
        assert!(buffer.dirty);
    }

    #[test]
    fn test_insert_chars_across_multiple_lines() {
        let mut buffer = Buffer::new_test_buffer("line1\nline2\nline3", 2);
        buffer.insert_char(Position::new(0, 2), 'A');
        buffer.insert_char(Position::new(1, 2), 'B');
        buffer.insert_char(Position::new(2, 2), 'C');
        assert_eq!(buffer.line_text(0), "liAne1");
        assert_eq!(buffer.line_text(1), "liBne2");
        assert_eq!(buffer.line_text(2), "liCne3");
    }

    #[test]
    fn test_insert_char_updates_buffer_length() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        let initial_len = buffer.len();
        buffer.insert_char(Position::new(0, 2), 'X');
        assert_eq!(buffer.len(), initial_len + 1);
    }

    #[test]
    fn test_insert_char_sets_dirty_flag() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        buffer.dirty = false;
        buffer.insert_char(Position::new(0, 2), 'X');
        assert!(buffer.dirty);
    }

    #[test]
    fn test_insert_char_invalid_line_position() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        let initial_text = buffer.line_text(0).to_string();
        buffer.insert_char(Position::new(10, 0), 'X');
        // Should not panic, buffer should remain unchanged
        assert_eq!(buffer.line_text(0), initial_text);
    }

    #[test]
    fn test_insert_special_characters() {
        let mut buffer = Buffer::new_test_buffer("hello", 2);
        buffer.insert_char(Position::new(0, 2), '€');
        assert_eq!(buffer.line_text(0), "he€llo");
        buffer.insert_char(Position::new(0, 3), '🚀');
        assert_eq!(buffer.line_text(0), "he€🚀llo");
        assert!(buffer.dirty);
    }

    #[test]
    fn test_get_text_single_line() {
        let buffer = Buffer::new_test_buffer("hello world", 2);
        let selection = Selection::new(Position::new(0, 0), Position::new(0, 5));
        assert_eq!(buffer.get_text(&selection), "hello");

        let selection = Selection::new(Position::new(0, 6), Position::new(0, 11));
        assert_eq!(buffer.get_text(&selection), "world");

        let selection = Selection::new(Position::new(0, 2), Position::new(0, 7));
        assert_eq!(buffer.get_text(&selection), "llo w");
    }

    #[test]
    fn test_get_text_two_lines() {
        let buffer = Buffer::new_test_buffer("hello\nworld", 2);
        let selection = Selection::new(Position::new(0, 2), Position::new(1, 3));
        assert_eq!(buffer.get_text(&selection), "llo\nwor");

        let selection = Selection::new(Position::new(0, 0), Position::new(1, 5));
        assert_eq!(buffer.get_text(&selection), "hello\nworld");
    }

    #[test]
    fn test_get_text_multiple_lines() {
        let buffer = Buffer::new_test_buffer("line1\nline2\nline3\nline4", 2);
        let selection = Selection::new(Position::new(0, 2), Position::new(3, 3));
        assert_eq!(buffer.get_text(&selection), "ne1\nline2\nline3\nlin");

        let selection = Selection::new(Position::new(1, 1), Position::new(2, 4));
        assert_eq!(buffer.get_text(&selection), "ine2\nline");
    }

    #[test]
    fn test_get_text_line_boundaries() {
        let buffer = Buffer::new_test_buffer("hello\nworld\ntest", 2);
        let selection = Selection::new(Position::new(0, 0), Position::new(0, 5));
        assert_eq!(buffer.get_text(&selection), "hello");

        let selection = Selection::new(Position::new(1, 0), Position::new(1, 5));
        assert_eq!(buffer.get_text(&selection), "world");

        let selection = Selection::new(Position::new(0, 5), Position::new(2, 0));
        assert_eq!(buffer.get_text(&selection), "\nworld\n");
    }

    #[test]
    fn test_get_text_special_characters_single_line() {
        let buffer = Buffer::new_test_buffer("hello €🚀 world", 2);
        let selection = Selection::new(Position::new(0, 6), Position::new(0, 9));
        assert_eq!(buffer.get_text(&selection), "€🚀 ");
    }

    #[test]
    fn test_get_text_special_characters_multiple_lines() {
        let buffer = Buffer::new_test_buffer("line1 €\nline2 🚀\nline3", 2);
        let selection = Selection::new(Position::new(0, 5), Position::new(2, 3));
        assert_eq!(buffer.get_text(&selection), " €\nline2 🚀\nlin");
    }
}
