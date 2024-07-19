use crate::textarea::textareaproperties::TextAreaProperties;

pub(crate) mod goto;

pub(crate) enum DirectTextAreaCommand {
    Goto,
}

impl TryFrom<&str> for DirectTextAreaCommand {
    type Error = String;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if command.starts_with(":") {
            Ok(Self::Goto)
        } else {
            Err("Invalid command".to_string())
        }
    }
}

impl DirectTextAreaCommand {
    pub(crate) fn execute(&self, command: &str, textarea: &mut TextAreaProperties) {
        match self {
            DirectTextAreaCommand::Goto => goto::execute(command, textarea),
        }
    }
}
