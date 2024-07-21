use crate::ceos::syntax::token_type::Token;
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

    pub(crate) fn token(&self) -> Option<&Token<'a>> {
        self.token.as_ref()
    }
}
