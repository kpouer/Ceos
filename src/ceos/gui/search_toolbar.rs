use egui;

#[derive(Debug, Default)]
pub(crate) struct SearchToolbar {
    pub(crate) query: String,
    pub(crate) case_sensitive: bool,
    pub(crate) whole_words: bool,
    pub(crate) is_regex: bool,
    pub(crate) should_focus: bool,
}

impl SearchToolbar {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui, open: &mut bool) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            let response = ui.text_edit_singleline(&mut self.query);

            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                *open = false;
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
}
