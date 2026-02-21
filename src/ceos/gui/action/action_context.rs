use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;

pub(crate) struct ActionContext<'a> {
    pub(crate) textarea_properties: &'a mut TextAreaProperties,
}

impl<'a> ActionContext<'a> {
    pub(crate) fn new(textarea_properties: &'a mut TextAreaProperties) -> Self {
        Self {
            textarea_properties,
        }
    }
}