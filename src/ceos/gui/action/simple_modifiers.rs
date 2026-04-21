use egui::Modifiers;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct SimpleModifiers {
    alt: bool,
    ctrl: bool,
    shift: bool,
    command: bool,
}

impl From<Modifiers> for SimpleModifiers {
    fn from(m: Modifiers) -> Self {
        Self {
            alt: m.alt,
            ctrl: m.ctrl,
            shift: m.shift,
            command: m.command,
        }
    }
}
