use crate::ceos::gui::action::Action;
use egui::{Key, KeyboardShortcut, Modifiers};
use log::debug;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct KeyboardHandler {
    shortcuts: HashMap<KeyboardShortcut, Action>,
}

impl KeyboardHandler {
    pub(crate) fn new() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert(KeyboardShortcut::new(Modifiers::CTRL, Key::S), Action::Save);
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::NONE, Key::ArrowLeft),
            Action::GoToPrevCharacter { select: false },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::SHIFT, Key::ArrowLeft),
            Action::GoToPrevCharacter { select: true },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::NONE, Key::ArrowRight),
            Action::GoToNextCharacter { select: false },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::SHIFT, Key::ArrowRight),
            Action::GoToNextCharacter { select: true },
        );

        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::NONE, Key::ArrowUp),
            Action::GoToPrevLine { select: false },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::SHIFT, Key::ArrowUp),
            Action::GoToPrevLine { select: true },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::NONE, Key::ArrowDown),
            Action::GoToNextLine { select: false },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::SHIFT, Key::ArrowDown),
            Action::GoToNextLine { select: true },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::NONE, Key::Enter),
            Action::Enter,
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::NONE, Key::Backspace),
            Action::Backspace,
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::NONE, Key::Delete),
            Action::Delete,
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::NONE, Key::Home),
            Action::GoToLineStart { select: false },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::NONE, Key::End),
            Action::GoToLineEnd { select: false },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::F7),
            Action::InsertHighlight,
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::F7),
            Action::InsertHighlight,
        );
        shortcuts.insert(
            KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::MAC_CMD | Modifiers::SHIFT,
                Key::F7,
            ),
            Action::InsertHighlight,
        );
        shortcuts.insert(KeyboardShortcut::new(Modifiers::CTRL, Key::Z), Action::Undo);
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::Z),
            Action::Redo,
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::CTRL, Key::Home),
            Action::GoToBufferStart { select: false },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::CTRL, Key::End),
            Action::GoToBufferEnd { select: false },
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::CTRL, Key::F),
            Action::Search,
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::CTRL, Key::R),
            Action::Replace,
        );
        shortcuts.insert(
            KeyboardShortcut::new(Modifiers::COMMAND, Key::Z),
            Action::Undo,
        );
        Self { shortcuts }
    }

    pub(crate) fn get_action(&self, keyboard_shortcut: &KeyboardShortcut) -> Option<&Action> {
        debug!("get_action {keyboard_shortcut:?}");
        self.shortcuts.get(keyboard_shortcut)
    }
}
