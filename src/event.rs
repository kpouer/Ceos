use crate::ceos::command::direct::goto::Goto;
use crate::ceos::command::direct::zoom::Zoom;
use crate::ceos::gui::textarea::buffer::Buffer;
use crate::event::Event::{BufferClosed, GotoLine, NewFont};
use egui::FontId;

pub(crate) enum Event {
    BufferLoaded(Buffer),
    BufferClosed,
    GotoLine(Goto),
    NewFont(FontId),
}

impl TryFrom<&str> for Event {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if command.starts_with(':') {
            if let Ok(goto) = Goto::try_from(command) {
                return Ok(GotoLine(goto));
            }
        } else if command == "close" {
            return Ok(BufferClosed);
        } else if command.starts_with("zoom ") {
            if let Ok(zoom) = Zoom::try_from(command) {
                return Ok(NewFont(zoom.get_font_id()));
            }
        }
        Err(())
    }
}
