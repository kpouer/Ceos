use std::collections::HashMap;
use egui::KeyboardShortcut;
use crate::ceos::gui::action::Action;

#[derive(Debug)]
pub(crate) struct KeyboardHandler {
    shortcuts: HashMap<KeyboardShortcut, Action>,
}

impl KeyboardHandler {
    pub(crate) fn new() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S), Action::Save);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowLeft), Action::GoToPrevCharacter);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowRight), Action::GoToNextCharacter);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowUp), Action::GoToPrevLine);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowDown), Action::GoToNextLine);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Enter), Action::Enter);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Backspace), Action::Backspace);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete), Action::Delete);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Home), Action::GoToLineStart);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::End), Action::GoToLineEnd);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Home), Action::GoToBufferStart);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::End), Action::GoToBufferEnd);
        shortcuts.insert(KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::F), Action::Search);
        Self { shortcuts }
    }

    pub(crate) fn get_action(&self, keyboard_shortcut: &KeyboardShortcut) -> Option<&Action> {
        self.shortcuts.get(keyboard_shortcut)
    }
}
