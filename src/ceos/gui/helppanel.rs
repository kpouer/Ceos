use egui::{Align2, Context, Grid, Key, Ui, Window};

pub(crate) struct HelpPanel;

impl HelpPanel {
    pub(crate) fn show(ctx: &Context, open: &mut bool) {
        Window::new("Commands help")
            .open(open)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Available commands :");
                    ui.add_space(8.0);
                    Grid::new("help_grid")
                        .spacing([20.0, 8.0])
                        .show(ui, |ui: &mut Ui| {
                            ui.label("?");
                            ui.label("Show this help panel");
                            ui.end_row();

                            ui.label(":<ligne>");
                            ui.label("Go to the specified line");
                            ui.end_row();

                            ui.label("s <text>");
                            ui.label("Search the given text");
                            ui.end_row();

                            ui.label("zoom <value>");
                            ui.label("Change the police size (ex: zoom 1.5, zoom reset)");
                            ui.end_row();

                            ui.label("filter <text>");
                            ui.label("Keep only the lines containing the given text. The text can be prefixed with ! to reverse the filter (use & for multiple conditions)");
                            ui.end_row();

                            ui.label("l <range>");
                            ui.label("Drop the lines within the range");
                            ui.label("ex: l ..10 will drop the lines 1 to 9");
                            ui.label("ex: l 5..10 will drop the lines 5 to 9");
                            ui.label("ex: l 10.. will drop the lines 10 to the end");
                            ui.end_row();

                            ui.label("close");
                            ui.label("Close the current file");
                            ui.end_row();
                        });
                    ui.add_space(16.0);
                    ui.label("Press 'Escape' to close this panel.");
                });
            });

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            *open = false;
        }
    }
}
