use crate::token::Token;
use logos::Span;

pub struct Chunk<'a> {
    pub(crate) token: Option<Token>,
    pub(crate) span: Span,
    text: &'a str,
}

impl<'a> Chunk<'a> {
    pub(crate) fn new(token: Option<Token>, span: Span, text: &'a str) -> Self {
        Self { token, span, text }
    }

    pub fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    pub fn as_str(&self) -> &str {
        &self.text[self.span.start..self.span.end]
    }

    pub fn start(&self) -> usize {
        self.span.start
    }

    pub(crate) fn merge(&mut self, chunk: &Chunk) {
        self.span.end = chunk.span.end;
    }
}
