use crate::ceos::command::direct::goto::Goto;
use crate::ceos::textarea::buffer::Buffer;
use crate::event::Event::{BufferClosed, GotoLine};

pub(crate) enum Event {
    BufferLoaded(Buffer),
    BufferClosed,
    GotoLine(Goto),
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
        }
        Err(())
    }
}
