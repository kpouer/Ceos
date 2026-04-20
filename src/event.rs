use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::command::direct::goto::Goto;
use crate::ceos::command::direct::zoom::Zoom;
use crate::ceos::highlight::highlight::Highlight;
use crate::event::Event::{BufferClosed, GotoLine, NewFont};
use crate::progress_operation::ProgressOperation;
use egui::FontId;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Event {
    /// BufferLoading(path, current, size)
    OpenFile(PathBuf),
    BufferLoadingStarted(PathBuf, u64),
    BufferLoading(PathBuf, u64, u64),
    // Saving progression events
    BufferSavingStarted(PathBuf, u64),
    BufferSaving(PathBuf, u64, u64),
    BufferSaved(PathBuf),
    BufferSaveFailed(PathBuf),
    BufferLoaded(Buffer),
    BufferClosed,
    GotoLine(Goto),
    NewFont(FontId),
    ShowHelp,
    SetCommand(String),
    /// Clear the current command
    ClearCommand,
    /// An operation started (label, total size)
    OperationStarted(ProgressOperation, usize),
    /// An operation is progressing (label, current status)
    OperationProgress(ProgressOperation, usize),
    /// An operation progress increment (label, amount)
    OperationIncrement(ProgressOperation, usize),
    /// An operation finished (label)
    OperationFinished(ProgressOperation),
    /// Show search bar
    ShowSearch,
    /// Show replace bar
    ShowReplace,
    /// Show Browser side panel
    ShowBrowser,
    /// Show Highlight side panel
    ShowHighlight,
    /// Add a highlight
    AddHighlight(Highlight),
    /// Remove a highlight by index
    RemoveHighlight(usize),
}

impl TryFrom<&str> for Event {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if command == "?" {
            return Ok(Event::ShowHelp);
        }
        if command.starts_with(':') {
            if let Ok(goto) = Goto::try_from(command) {
                return Ok(GotoLine(goto));
            }
        } else if command == "close" {
            return Ok(BufferClosed);
        } else if let Some(stripped) = command.strip_prefix("highlight ") {
            let parts: Vec<&str> = stripped.split_whitespace().collect();
            if !parts.is_empty() {
                let text = parts[0].to_string();
                let case_insensitive = parts.get(1).map(|&s| s == "i").unwrap_or(false);
                let color = crate::ceos::highlight::deterministic_color(&text);

                return Ok(Event::AddHighlight(Highlight::new(
                    text,
                    case_insensitive,
                    color,
                )));
            }
        } else if command.starts_with("zoom ")
            && let Ok(zoom) = Zoom::try_from(command)
        {
            return Ok(NewFont(zoom.get_font_id()));
        }
        Err(())
    }
}
