use crate::event::Event;
use log::info;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(crate) mod action_context;
pub(crate) mod keyboard_handler;
pub(crate) mod shortcut_parser;
mod simple_modifiers;
mod simple_shortcut;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Save,
    GoToPrevCharacter { select: bool },
    GoToNextCharacter { select: bool },
    GoToPrevLine { select: bool },
    GoToNextLine { select: bool },
    GoToLineStart { select: bool },
    GoToLineEnd { select: bool },
    GoToBufferStart { select: bool },
    GoToBufferEnd { select: bool },
    InsertHighlight,
    Enter,
    Backspace,
    Delete,
    Search,
    Replace,
    Undo,
    Redo,
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let name = match self {
            Action::Save => "Save",
            Action::GoToPrevCharacter { select: false } => "GoToPrevCharacter",
            Action::GoToPrevCharacter { select: true } => "GoToPrevCharacterSelect",
            Action::GoToNextCharacter { select: false } => "GoToNextCharacter",
            Action::GoToNextCharacter { select: true } => "GoToNextCharacterSelect",
            Action::GoToPrevLine { select: false } => "GoToPrevLine",
            Action::GoToPrevLine { select: true } => "GoToPrevLineSelect",
            Action::GoToNextLine { select: false } => "GoToNextLine",
            Action::GoToNextLine { select: true } => "GoToNextLineSelect",
            Action::GoToLineStart { select: false } => "GoToLineStart",
            Action::GoToLineStart { select: true } => "GoToLineStartSelect",
            Action::GoToLineEnd { select: false } => "GoToLineEnd",
            Action::GoToLineEnd { select: true } => "GoToLineEndSelect",
            Action::GoToBufferStart { select: false } => "GoToBufferStart",
            Action::GoToBufferStart { select: true } => "GoToBufferStartSelect",
            Action::GoToBufferEnd { select: false } => "GoToBufferEnd",
            Action::GoToBufferEnd { select: true } => "GoToBufferEndSelect",
            Action::InsertHighlight => "InsertHighlight",
            Action::Enter => "Enter",
            Action::Backspace => "Backspace",
            Action::Delete => "Delete",
            Action::Search => "Search",
            Action::Replace => "Replace",
            Action::Undo => "Undo",
            Action::Redo => "Redo",
        };
        serializer.serialize_str(name)
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Save" => Ok(Action::Save),
            "GoToPrevCharacter" => Ok(Action::GoToPrevCharacter { select: false }),
            "GoToPrevCharacterSelect" => Ok(Action::GoToPrevCharacter { select: true }),
            "GoToNextCharacter" => Ok(Action::GoToNextCharacter { select: false }),
            "GoToNextCharacterSelect" => Ok(Action::GoToNextCharacter { select: true }),
            "GoToPrevLine" => Ok(Action::GoToPrevLine { select: false }),
            "GoToPrevLineSelect" => Ok(Action::GoToPrevLine { select: true }),
            "GoToNextLine" => Ok(Action::GoToNextLine { select: false }),
            "GoToNextLineSelect" => Ok(Action::GoToNextLine { select: true }),
            "GoToLineStart" => Ok(Action::GoToLineStart { select: false }),
            "GoToLineStartSelect" => Ok(Action::GoToLineStart { select: true }),
            "GoToLineEnd" => Ok(Action::GoToLineEnd { select: false }),
            "GoToLineEndSelect" => Ok(Action::GoToLineEnd { select: true }),
            "GoToBufferStart" => Ok(Action::GoToBufferStart { select: false }),
            "GoToBufferStartSelect" => Ok(Action::GoToBufferStart { select: true }),
            "GoToBufferEnd" => Ok(Action::GoToBufferEnd { select: false }),
            "GoToBufferEndSelect" => Ok(Action::GoToBufferEnd { select: true }),
            "InsertHighlight" => Ok(Action::InsertHighlight),
            "Enter" => Ok(Action::Enter),
            "Backspace" => Ok(Action::Backspace),
            "Delete" => Ok(Action::Delete),
            "Search" => Ok(Action::Search),
            "Replace" => Ok(Action::Replace),
            "Undo" => Ok(Action::Undo),
            "Redo" => Ok(Action::Redo),
            _ => Err(serde::de::Error::custom(format!("Unknown action: {}", s))),
        }
    }
}

impl Action {
    pub(crate) fn execute(&self, context: &mut action_context::ActionContext) {
        match self {
            Action::Backspace => context.textarea_properties.input_backspace(),
            Action::Delete => context.textarea_properties.input_delete(),
            Action::Enter => context.textarea_properties.input_enter(),
            Action::GoToPrevCharacter { select } => {
                context.textarea_properties.go_to_prev_char(*select)
            }
            Action::GoToNextCharacter { select } => {
                context.textarea_properties.go_to_next_char(*select)
            }
            Action::GoToPrevLine { select } => context.textarea_properties.go_to_prev_line(*select),
            Action::GoToNextLine { select } => context.textarea_properties.go_to_next_line(*select),
            Action::GoToLineStart { select } => {
                context.textarea_properties.go_to_start_of_line(*select)
            }
            Action::GoToLineEnd { select } => {
                context.textarea_properties.go_to_end_of_line(*select)
            }
            Action::GoToBufferStart { select } => {
                context.textarea_properties.go_to_start_of_buffer(*select)
            }
            Action::GoToBufferEnd { select } => {
                context.textarea_properties.go_to_end_of_buffer(*select)
            }
            Action::Save => info!("Save action triggered"),
            Action::Undo => context.textarea_properties.undo(),
            Action::Redo => context.textarea_properties.redo(),
            Action::Search => {
                let _ = context
                    .textarea_properties
                    .buffer
                    .sender
                    .send(Event::ShowSearch);
            }
            Action::Replace => {
                let _ = context
                    .textarea_properties
                    .buffer
                    .sender
                    .send(Event::ShowReplace);
            }
            Action::InsertHighlight => {
                context.textarea_properties.insert_highlight();
            }
        }
    }
}
