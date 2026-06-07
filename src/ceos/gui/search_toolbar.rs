use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::gui::textpane::caret_position::CaretPosition;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use buffer_core::position::Position;
use buffer_core::text_range::Selection;
use egui;
use egui::Ui;
use log::info;
use search::SearchMatcher;
use search::regex_search_matcher::RegexSearchMatcher;
use search::simple_search_case_sensitive::SimpleSearchCaseSensitiveMatcher;
use search::simple_search_matcher_case_insensitive::SimpleSearchCaseInsensitiveMatcher;

#[derive(Debug, Default)]
pub(crate) struct SearchToolbar {
    pub(crate) query: String,
    pub(crate) replacement: String,
    pub(crate) case_sensitive: bool,
    pub(crate) whole_words: bool,
    pub(crate) is_regex: bool,
    pub(crate) show_replace: bool,
    pub(crate) should_focus: bool,
    pub(crate) last_search_failed: bool,
    pub(crate) start_search_pos: Option<Position>,
    search_matcher: Option<Box<dyn SearchMatcher>>,
}

impl SearchToolbar {
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        open: &mut bool,
        textarea_properties: &mut TextAreaProperties,
    ) {
        if self.start_search_pos.is_none() {
            self.start_search_pos = Some(textarea_properties.caret_position.position);
        }

        ui.vertical(|ui| {
            self.show_search(open, textarea_properties, ui);

            if self.show_replace {
                self.show_replace(textarea_properties, ui);
            }
        });
    }

    fn show_search(
        &mut self,
        open: &mut bool,
        textarea_properties: &mut TextAreaProperties,
        ui: &mut Ui,
    ) {
        ui.horizontal(|ui| {
            ui.label("Search: ");
            let search_text_response = ui.text_edit_singleline(&mut self.query);

            self.show_search_options_buttons(
                textarea_properties,
                ui,
                search_text_response.changed(),
            );

            if self.last_search_failed {
                ui.label(egui::RichText::new("No results").color(egui::Color32::RED));
            }

            if search_text_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                *open = false;
                self.start_search_pos = None;
            }

            if search_text_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.do_search(textarea_properties);
            }

            if self.should_focus {
                search_text_response.request_focus();
                self.should_focus = false;
            }

            ui.allocate_ui(ui.available_size(), |ui: &mut egui::Ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("X").clicked() {
                        *open = false;
                        self.start_search_pos = None;
                    }
                });
            });
        });
    }

    fn show_replace(&mut self, textarea_properties: &mut TextAreaProperties, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Replace:");
            ui.text_edit_singleline(&mut self.replacement);
            if ui.button("Replace").clicked() {
                textarea_properties.replace_selection(&self.replacement);
                self.do_search(textarea_properties);
            }
            if ui.button("Replace all").clicked() {
                // Logique de remplacement global à ajouter
            }
        });
    }

    fn show_search_options_buttons(
        &mut self,
        textarea_properties: &mut TextAreaProperties,
        ui: &mut Ui,
        search_text_modified: bool,
    ) {
        if ui
            .toggle_value(&mut self.case_sensitive, "Cc")
            .on_hover_text("Case sensitive")
            .changed()
            || ui
                .toggle_value(&mut self.whole_words, "W")
                .on_hover_text("Entire words")
                .changed()
            || ui
                .toggle_value(&mut self.is_regex, ".*")
                .on_hover_text("Regular expression")
                .changed()
            || search_text_modified
        {
            self.last_search_failed = false;
            self.search_matcher = self.build_search_matcher().ok();
            self.do_search_from_start(textarea_properties);
        }
    }

    fn do_search_from_start(&mut self, textarea_properties: &mut TextAreaProperties) {
        let start_pos = self.start_search_pos.unwrap_or(Position::ZERO);
        self.do_search_inner(textarea_properties, start_pos, false)
    }

    fn do_search(&mut self, textarea_properties: &mut TextAreaProperties) {
        let start_pos = textarea_properties.caret_position.position;
        self.do_search_inner(textarea_properties, start_pos, true)
    }

    fn do_search_inner(
        &mut self,
        textarea_properties: &mut TextAreaProperties,
        start_pos: Position,
        find_next: bool,
    ) {
        info!("do_search_inner {self:?} start_pos: {start_pos} find_next: {find_next}");
        self.last_search_failed = false;
        if self.query.is_empty() {
            return;
        }

        let line_count = textarea_properties.buffer.line_count();
        if line_count == 0 {
            return;
        }

        // Search from current position to end of buffer
        for line_idx in start_pos.line..line_count {
            let from_col = if line_idx == start_pos.line {
                if find_next {
                    start_pos.column + 1
                } else {
                    start_pos.column
                }
            } else {
                0
            };

            if let Some((start, end)) =
                self.find_in_line(&mut textarea_properties.buffer, line_idx, from_col)
            {
                self.apply_found_match(textarea_properties, line_idx, start, end);
                return;
            }
        }

        // Wrap around: search from start of buffer to current position
        for line_idx in 0..=start_pos.line {
            let to_col = if line_idx == start_pos.line {
                if find_next {
                    start_pos.column
                } else {
                    // if not find_next, we already searched the whole line above starting from start_pos.column
                    // so we only need to search before start_pos.column?
                    // actually if we didn't find it from start_pos.column to end,
                    // we search from 0 to start_pos.column
                    start_pos.column.saturating_sub(1)
                }
            } else {
                textarea_properties.buffer.line_text(line_idx).len()
            };

            if let Some((start, end)) =
                self.find_in_line(&mut textarea_properties.buffer, line_idx, 0)
                && (line_idx < start_pos.line || start <= to_col)
            {
                self.apply_found_match(textarea_properties, line_idx, start, end);
                return;
            }
        }
        self.last_search_failed = true;
    }

    fn build_search_matcher(&self) -> Result<Box<dyn SearchMatcher>, ()> {
        Ok(if self.is_regex {
            Box::new(
                RegexSearchMatcher::new(&self.query, self.case_sensitive, self.whole_words)
                    .map_err(|_| ())?,
            )
        } else if self.case_sensitive {
            Box::new(SimpleSearchCaseSensitiveMatcher::new(
                &self.query,
                self.whole_words,
            ))
        } else {
            Box::new(SimpleSearchCaseInsensitiveMatcher::new(
                &self.query,
                self.whole_words,
            ))
        })
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
        let end = CaretPosition {
            position: Position {
                line,
                column: end_col,
            },
            virtual_column: 0,
        };
        textarea_properties.caret_position = end;
        textarea_properties.set_selection(Selection::new(start, end.position));
        // Simple scroll to make it visible
        textarea_properties.set_first_line(line.saturating_sub(5));
    }

    fn find_in_line(
        &self,
        buffer: &mut Buffer,
        line_idx: usize,
        from_col: usize,
    ) -> Option<(usize, usize)> {
        let Some(search_matcher) = &self.search_matcher else {
            return None;
        };
        let line_text = buffer.line_text_with_decompress(line_idx);

        if from_col >= line_text.len() && from_col > 0 {
            return None;
        }

        search_matcher.search(line_text, from_col)
    }
}
