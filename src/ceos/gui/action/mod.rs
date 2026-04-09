use crate::event::Event;
use log::info;

pub(crate) mod action_context;
pub(crate) mod keyboard_handler;

#[derive(Debug, Clone, Copy)]
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
    Enter,
    Backspace,
    Delete,
    Search,
    Replace,
    Undo,
    Redo,
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
        }
    }
}
