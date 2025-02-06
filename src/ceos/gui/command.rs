use egui::{Response, Ui, Widget};

pub(crate) struct Command<'a> {
    command_buffer: &'a mut String,
}

impl<'a> Command<'a> {
    pub(crate) fn show(command_buffer: &'a mut String, ui: &mut Ui) -> Response {
        Self { command_buffer }.ui(ui)
    }
}

impl Widget for Command<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.label("Command: ");
        let response = ui.add_sized(
            ui.available_size(),
            egui::TextEdit::singleline(self.command_buffer),
        );
        ui.memory_mut(|memory| {
            memory.request_focus(response.id);
        });
        response
    }
}