use std::fmt::{Debug, Display};

use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::gui::textpane::renderer::Renderer;

pub(crate) mod direct;
pub(crate) mod filter;
pub(crate) mod search;
pub(crate) mod save_action;

pub(crate) trait Action: Debug {
    fn execute(&self, buffer: &mut Buffer);
}

pub(crate) trait Command: Action + Renderer + Display {
}
