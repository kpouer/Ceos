use egui::{Response, Ui, Widget};

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum DockType {
    Browser,
    Highlight,
}

#[derive(Debug)]
pub(crate) struct DockStatus {
    current_dock: Option<DockType>,
    pub(crate) side_panel_width: f32,
}

impl DockStatus {
    pub(crate) fn toggle(&mut self, dock_type: DockType) {
        if self.current_dock.as_ref() == Some(&dock_type) {
            self.current_dock = None;
        } else {
            self.current_dock = Some(dock_type);
        }
    }

    pub(crate) fn current(&self) -> Option<DockType> {
        self.current_dock
    }
}

impl Default for DockStatus {
    fn default() -> Self {
        Self {
            current_dock: None,
            side_panel_width: 200.0,
        }
    }
}

impl Widget for DockStatus {
    fn ui(self, ui: &mut Ui) -> Response {
        todo!()
    }
}
