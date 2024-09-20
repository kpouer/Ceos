use crate::ceos::buffer::Buffer;
use crate::ceos::command::search::Search;
use egui::{Label, ScrollArea, TextWrapMode};
use egui_extras::{Column, TableBuilder};

pub(crate) fn build_search_panel(buffer: &Buffer, ui: &mut egui::Ui, search: &Search) {
    ScrollArea::both().show(ui, |ui| {
        TableBuilder::new(ui)
            .column(Column::auto().resizable(true))
            .column(Column::remainder())
            .body(|body| {
                body.rows(30.0, search.lines.len(), |mut row| {
                    let row_index = row.index();
                    let line_number = search.lines[row_index];
                    let line_label =
                        Label::new(line_number.to_string()).wrap_mode(TextWrapMode::Extend);
                    let line_text_label =
                        Label::new(buffer.line_text(line_number)).wrap_mode(TextWrapMode::Extend);
                    row.col(|ui| {
                        ui.add(line_label);
                    });
                    row.col(|ui| {
                        ui.add(line_text_label);
                    });
                });
            });
    });
}
