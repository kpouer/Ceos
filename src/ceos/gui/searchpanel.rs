use crate::ceos::buffer::Buffer;
use crate::ceos::command::direct::goto::Goto;
use crate::ceos::command::search::Search;
use crate::event::Event;
use crate::event::Event::GotoLine;
use egui::{Label, ScrollArea, Sense, TextWrapMode};
use egui_extras::{Column, TableBuilder, TableRow};
use std::sync::mpsc::Sender;

pub(crate) struct SearchPanel {
    pub(crate) search: Search,
    sender: Sender<Event>,
}

impl SearchPanel {
    pub(crate) fn new(sender: Sender<Event>) -> Self {
        Self {
            search: Default::default(),
            sender,
        }
    }

    pub(crate) fn ui(&self, buffer: &Buffer, ui: &mut egui::Ui) {
        ScrollArea::both().show(ui, |ui| {
            TableBuilder::new(ui)
                .sense(Sense::click())
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
                .body(|body| {
                    body.rows(30.0, self.search.lines.len(), |mut row| {
                        let row_index = row.index();
                        let line_number = self.search.lines[row_index];
                        self.add_row(&mut row, line_number, line_number.to_string());
                        self.add_row(
                            &mut row,
                            line_number,
                            buffer.line_text(line_number).to_string(),
                        );
                    });
                });
        });
    }

    fn add_row(&self, row: &mut TableRow, line_number: usize, text: String) {
        let label = Label::new(text)
            .wrap_mode(TextWrapMode::Extend)
            .selectable(false);
        row.col(|ui| {
            ui.add(label);
        })
        .1
        .clicked()
        .then(|| self.sender.send(GotoLine(Goto::from(line_number))).unwrap());
    }
}
