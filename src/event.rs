use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::command::direct::goto::Goto;
use crate::ceos::command::direct::zoom::Zoom;
use crate::event::Event::{BufferClosed, GotoLine, NewFont};
use egui::FontId;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum Event {
    /// BufferLoading(path, current, size)
    OpenFile(PathBuf),
    BufferLoadingStarted(PathBuf, usize),
    BufferLoading(PathBuf, usize, usize),
    // Saving progression events
    BufferSavingStarted(PathBuf, usize),
    BufferSaving(PathBuf, usize, usize),
    BufferSaved(PathBuf),
    BufferSaveFailed(PathBuf),
    BufferLoaded(Buffer),
    BufferClosed,
    GotoLine(Goto),
    NewFont(FontId),
    SetCommand(String),
    /// Clear the current command
    ClearCommand,
    /// An operation started (label, total size)
    OperationStarted(String, usize),
    /// An operation is progressing (label, current status)
    OperationProgress(String, usize),
    /// An operation progress increment (label, amount)
    OperationIncrement(String, usize),
    /// An operation finished (label)
    OperationFinished(String),
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
        } else if command.starts_with("zoom ")
            && let Ok(zoom) = Zoom::try_from(command)
        {
            return Ok(NewFont(zoom.get_font_id()));
        }
        Err(())
    }
}
