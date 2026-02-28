use egui;

#[derive(Debug)]
pub(crate) struct SearchWidget {
    pub(crate) query: String,
    pub(crate) case_sensitive: bool,
    pub(crate) whole_words: bool,
    pub(crate) is_regex: bool,
    pub(crate) should_focus: bool,
}

impl SearchWidget {
    pub(crate) fn new() -> Self {
        Self {
            query: String::new(),
            case_sensitive: false,
            whole_words: false,
            is_regex: false,
            should_focus: false,
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut egui::Ui, open: &mut bool) {
        ui.horizontal(|ui| {
            ui.label("Chercher:");
            let response = ui.text_edit_singleline(&mut self.query);

            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                *open = false;
            }

            if self.should_focus {
                response.request_focus();
                self.should_focus = false;
            }

            ui.toggle_value(&mut self.case_sensitive, "Cc")
                .on_hover_text("Respecter la casse");
            ui.toggle_value(&mut self.whole_words, "W")
                .on_hover_text("Mots entiers");
            ui.toggle_value(&mut self.is_regex, ".*")
                .on_hover_text("Expression régulière");

            if ui.button("X").clicked() {
                *open = false;
            }
        });
    }
}
