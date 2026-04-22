use crate::ceos::gui::action::simple_modifiers::SimpleModifiers;
use egui::{Key, Modifiers};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct SimpleShortcut {
    modifiers: SimpleModifiers,
    key: Key,
}

impl SimpleShortcut {
    pub(crate) fn new(modifiers: Modifiers, key: Key) -> Self {
        Self {
            modifiers: SimpleModifiers::from(modifiers),
            key,
        }
    }
}
