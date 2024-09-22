use crate::ceos::buffer::Buffer;
use crate::ceos::command::direct::goto::Goto;
use crate::ceos::command::search::Search;
use crate::event::Event;
use crate::event::Event::GotoLine;
use egui::{Label, ScrollArea, Sense, TextWrapMode};
use egui_extras::{Column, TableBuilder, TableRow};
use std::sync::mpsc::Sender;

pub(crate) fn build_search_panel(
    sender: &Sender<Event>,
    buffer: &Buffer,
    ui: &mut egui::Ui,
    search: &Search,
) {
    ScrollArea::both().show(ui, |ui| {
        let table = TableBuilder::new(ui)
            .sense(Sense::click())
            .column(Column::auto().resizable(true))
            .column(Column::remainder());
        table.body(|body| {
            body.rows(30.0, search.lines.len(), |mut row| {
                let row_index = row.index();
                let line_number = search.lines[row_index];
                add_row(sender, &mut row, line_number, line_number.to_string());
                add_row(
                    sender,
                    &mut row,
                    line_number,
                    buffer.line_text(line_number).to_string(),
                );
            });
        });
    });
}

fn add_row(sender: &Sender<Event>, row: &mut TableRow, line_number: usize, text: String) {
    let label = Label::new(text)
        .wrap_mode(TextWrapMode::Extend)
        .selectable(false);
    row.col(|ui| {
        ui.add(label);
    })
    .1
    .clicked()
    .then(|| {
        sender.send(GotoLine(Goto::from(line_number))).unwrap();
    });
}
