use crate::ceos::syntax::token_type::Token;
use eframe::epaint::Color32;
use egui::Visuals;

mod jedit;
pub(crate) mod solarized;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Theme {
    pub(crate) dark: bool,
    pub(crate) background: Color32,
    pub(crate) background_faint: Color32,
    pub(crate) text: Color32,
    pub(crate) text_faint: Color32,
    pub(crate) info: Color32,
    pub(crate) warning: Color32,
    pub(crate) error: Color32,
    pub(crate) literal: Color32,
    pub(crate) operator: Color32,
    pub(crate) number: Color32,
    pub(crate) string: Color32,
    pub(crate) deleting: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::solarized_dark()
    }
}

impl From<&Theme> for Visuals {
    fn from(theme: &Theme) -> Visuals {
        let mut visuals = if theme.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        visuals.panel_fill = theme.background;
        visuals.extreme_bg_color = theme.background;
        visuals.faint_bg_color = theme.background_faint;
        visuals.override_text_color = Some(theme.text);
        visuals
    }
}

impl Theme {
    pub(crate) fn color(&self, token: &Token) -> Color32 {
        match token {
            Token::Bool(_) => self.literal,
            Token::BraceOpen => self.operator,
            Token::BraceClose => self.operator,
            Token::BracketOpen => self.operator,
            Token::BracketClose => self.operator,
            Token::Colon => self.operator,
            Token::Comma => self.operator,
            Token::Null => self.literal,
            Token::Number(_) => self.number,
            Token::String(_) => self.string,
            Token::Info => self.info,
            Token::Warning => self.warning,
            Token::Error => self.error,
            Token::Fatal => self.error,
        }
    }
}
