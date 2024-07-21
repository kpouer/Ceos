use crate::syntax::token_type::Token;
use eframe::epaint::Color32;
use logos::Span;

pub(crate) struct Chunk<'a> {
    token: Option<Token<'a>>,
    span: Span,
    text: &'a str,
}

impl<'a> Chunk<'a> {
    pub(crate) fn new(token: Option<Token<'a>>, span: Span, text: &'a str) -> Self {
        Self { token, span, text }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text[self.span.start..self.span.end]
    }

    pub(crate) fn start(&self) -> usize {
        self.span.start
    }

    pub(crate) fn color(&self) -> Option<Color32> {
        self.token.as_ref().map(|token| token.get_color())
    }
}
