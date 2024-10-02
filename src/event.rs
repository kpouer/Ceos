use crate::ceos::buffer::Buffer;
use crate::ceos::command::direct::goto::Goto;
use crate::ceos::command::direct::zoom::Zoom;
use crate::event::Event::{BufferClosed, GotoLine, NewFont};
use egui::FontId;
use std::path::PathBuf;

pub(crate) enum Event {
    /// BufferLoading(path, current, size)
    OpenFile(PathBuf),
    BufferLoadingStarted(PathBuf, usize),
    BufferLoading(PathBuf, usize, usize),
    BufferLoaded(Buffer),
    BufferClosed,
    GotoLine(Goto),
    NewFont(FontId),
    SetCommand(String),
    ClearCommand,
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
