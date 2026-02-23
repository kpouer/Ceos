use log::info;

pub(crate) mod action_context;
pub(crate) mod keyboard_handler;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Action {
    Save,
    GoToPrevCharacter,
    GoToNextCharacter,
    GoToPrevLine,
    GoToNextLine,
    GoToLineStart,
    GoToLineEnd,
    GoToBufferStart,
    GoToBufferEnd,
    Enter,
    Backspace,
    Delete,
}

impl Action {
    pub(crate) fn execute(&self, context: &mut action_context::ActionContext) {
        match self {
            Action::Backspace => context.textarea_properties.input_backspace(),
            Action::Delete => context.textarea_properties.input_backspace(),
            Action::Enter => context.textarea_properties.input_enter(),
            Action::GoToPrevCharacter => context.textarea_properties.go_to_prev_char(),
            Action::GoToNextCharacter => context.textarea_properties.go_to_next_char(),
            Action::GoToPrevLine => context.textarea_properties.go_to_prev_line(),
            Action::GoToNextLine => context.textarea_properties.go_to_next_line(),
            Action::GoToLineStart => context.textarea_properties.go_to_start_of_line(),
            Action::GoToLineEnd => context.textarea_properties.go_to_end_of_line(),
            Action::GoToBufferStart => context.textarea_properties.go_to_start_of_buffer(),
            Action::GoToBufferEnd => context.textarea_properties.go_to_end_of_buffer(),
            Action::Save => info!("Save action triggered"),
        }
    }
}
