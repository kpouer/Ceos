use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::selection::Selection;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::search::SearchMatcher;
use crate::ceos::search::regex_search_matcher::RegexSearchMatcher;
use crate::ceos::search::simple_search_matcher::SimpleSearchMatcher;
use egui;
use log::info;

#[derive(Debug, Default)]
pub(crate) struct SearchToolbar {
    pub(crate) query: String,
    pub(crate) case_sensitive: bool,
    pub(crate) whole_words: bool,
    pub(crate) is_regex: bool,
    pub(crate) should_focus: bool,
}

impl SearchToolbar {
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        open: &mut bool,
        textarea_properties: &mut TextAreaProperties,
    ) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            let response = ui.text_edit_singleline(&mut self.query);

            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                *open = false;
            }

            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let _ = self.do_search(textarea_properties);
            }

            if self.should_focus {
                response.request_focus();
                self.should_focus = false;
            }

            ui.toggle_value(&mut self.case_sensitive, "Cc")
                .on_hover_text("Case sensitive");
            ui.toggle_value(&mut self.whole_words, "W")
                .on_hover_text("Entire words");
            ui.toggle_value(&mut self.is_regex, ".*")
                .on_hover_text("Regular expression");

            ui.allocate_ui(ui.available_size(), |ui: &mut egui::Ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("X").clicked() {
                        *open = false;
                    }
                });
            });
        });
    }

    fn do_search(&self, textarea_properties: &mut TextAreaProperties) -> Result<(), ()> {
        info!("do_search {self:?}");
        if self.query.is_empty() {
            return Err(());
        }

        let buffer = &textarea_properties.buffer;
        let line_count = buffer.line_count();
        if line_count == 0 {
            return Err(());
        }

        let start_pos = textarea_properties.caret_position;

        let search_matcher: Box<dyn SearchMatcher> = if self.is_regex {
            Box::new(
                RegexSearchMatcher::new(&self.query, self.case_sensitive, self.whole_words)
                    .map_err(|e| ())?,
            )
        } else {
            Box::new(SimpleSearchMatcher::new(
                &self.query,
                self.case_sensitive,
                self.whole_words,
            ))
        };

        let find_in_line = |line_idx: usize, from_col: usize| -> Option<(usize, usize)> {
            let line_text = buffer.line_text(line_idx);

            if from_col >= line_text.len() && from_col > 0 {
                return None;
            }

            search_matcher.search(line_text, from_col)
        };

        // Search from current position to end of buffer
        for line_idx in start_pos.line..line_count {
            let from_col = if line_idx == start_pos.line {
                start_pos.column + 1
            } else {
                0
            };

            if let Some((start, end)) = find_in_line(line_idx, from_col) {
                self.apply_found_match(textarea_properties, line_idx, start, end);
                return Err(());
            }
        }

        // Wrap around: search from start of buffer to current position
        for line_idx in 0..=start_pos.line {
            let to_col = if line_idx == start_pos.line {
                start_pos.column
            } else {
                buffer.line_text(line_idx).len()
            };

            if let Some((start, end)) = find_in_line(line_idx, 0) {
                if line_idx < start_pos.line || start <= to_col {
                    self.apply_found_match(textarea_properties, line_idx, start, end);
                    return Err(());
                }
            }
        }
        Ok(())
    }

    fn apply_found_match(
        &self,
        textarea_properties: &mut TextAreaProperties,
        line: usize,
        start_col: usize,
        end_col: usize,
    ) {
        let start = Position {
            line,
            column: start_col,
        };
        let end = Position {
            line,
            column: end_col,
        };
        textarea_properties.caret_position = end;
        textarea_properties.selection = Some(Selection::new(start, end));
        // Simple scroll to make it visible
        textarea_properties.set_first_line(line.saturating_sub(5));
    }
}
