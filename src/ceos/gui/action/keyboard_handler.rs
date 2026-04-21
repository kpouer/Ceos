use crate::ceos::gui::action::Action;
use crate::ceos::gui::action::simple_shortcut::SimpleShortcut;
use egui::{Key, KeyboardShortcut, Modifiers};
use log::debug;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct KeyboardHandler {
    shortcuts: HashMap<SimpleShortcut, Action>,
}

impl KeyboardHandler {
    pub(crate) fn new() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert(SimpleShortcut::new(Modifiers::CTRL, Key::S), Action::Save);
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::NONE, Key::ArrowLeft),
            Action::GoToPrevCharacter { select: false },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::SHIFT, Key::ArrowLeft),
            Action::GoToPrevCharacter { select: true },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::NONE, Key::ArrowRight),
            Action::GoToNextCharacter { select: false },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::SHIFT, Key::ArrowRight),
            Action::GoToNextCharacter { select: true },
        );

        shortcuts.insert(
            SimpleShortcut::new(Modifiers::NONE, Key::ArrowUp),
            Action::GoToPrevLine { select: false },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::SHIFT, Key::ArrowUp),
            Action::GoToPrevLine { select: true },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::NONE, Key::ArrowDown),
            Action::GoToNextLine { select: false },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::SHIFT, Key::ArrowDown),
            Action::GoToNextLine { select: true },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::NONE, Key::Enter),
            Action::Enter,
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::NONE, Key::Backspace),
            Action::Backspace,
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::NONE, Key::Delete),
            Action::Delete,
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::NONE, Key::Home),
            Action::GoToLineStart { select: false },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::NONE, Key::End),
            Action::GoToLineEnd { select: false },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::F7),
            Action::InsertHighlight,
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::F7),
            Action::InsertHighlight,
        );
        shortcuts.insert(
            SimpleShortcut::new(
                Modifiers::COMMAND | Modifiers::MAC_CMD | Modifiers::SHIFT,
                Key::F7,
            ),
            Action::InsertHighlight,
        );
        shortcuts.insert(SimpleShortcut::new(Modifiers::CTRL, Key::Z), Action::Undo);
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::Z),
            Action::Redo,
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::CTRL, Key::Home),
            Action::GoToBufferStart { select: false },
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::CTRL, Key::End),
            Action::GoToBufferEnd { select: false },
        );
        shortcuts.insert(SimpleShortcut::new(Modifiers::CTRL, Key::F), Action::Search);
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::CTRL, Key::R),
            Action::Replace,
        );
        shortcuts.insert(
            SimpleShortcut::new(Modifiers::COMMAND, Key::Z),
            Action::Undo,
        );
        Self { shortcuts }
    }

    pub(crate) fn get_action(&self, keyboard_shortcut: &KeyboardShortcut) -> Option<&Action> {
        debug!("get_action {keyboard_shortcut:?}");
        let simple_shortcut =
            SimpleShortcut::new(keyboard_shortcut.modifiers, keyboard_shortcut.logical_key);
        self.shortcuts.get(&simple_shortcut)
    }
}
