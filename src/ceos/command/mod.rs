use std::fmt::Display;

use crate::ceos::buffer::Buffer;
use crate::ceos::gui::textpane::renderer::Renderer;

pub(crate) mod direct;
pub(crate) mod filter;
pub(crate) mod search;

pub(crate) trait Command: Renderer + Display {
    fn execute(&self, buffer: &mut Buffer);
}
