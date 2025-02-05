use crate::ceos::model::highlight::Highlight;
use eframe::epaint::text::TextWrapMode;
use egui::{Label, ScrollArea, Sense, Widget};
use egui_extras::{Column, TableBuilder, TableRow};

struct HighlightPanel<'a> {
    highlights: &'a Vec<Highlight>,
}

impl HighlightPanel<'_> {
    fn ui(self, ui: &mut egui::Ui) {
        ScrollArea::both().show(ui, |ui| {
            TableBuilder::new(ui)
                .sense(Sense::click())
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
                .body(|body| {
                    body.rows(30.0, self.highlights.len(), |mut row| {
                        let row_index = row.index();
                        let highlight = &self.highlights[row_index];
                        self.add_row(&mut row, highlight);
                    });
                });
        });
    }

    fn add_row(&self, row: &mut TableRow, highlight: &Highlight) {
        let label = Label::new(highlight.pattern().to_string())
            .wrap_mode(TextWrapMode::Extend)
            .selectable(false);
        row.col(|ui| {
            ui.add(label);
        })
        .1;
        // .clicked()
        // .then(|| self.sender.send(GotoLine(Goto::from(line_number))).unwrap());
    }
}
