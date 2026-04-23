use crate::token::Token;
use logos::Span;

#[derive(Debug)]
pub struct Chunk<'a> {
    pub token: Option<Token>,
    span: Span,
    text: &'a str,
}

impl<'a> Chunk<'a> {
    #[inline]
    pub const fn new(token: Option<Token>, span: Span, text: &'a str) -> Self {
        Self { token, span, text }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.text[self.span.start..self.span.end]
    }

    #[inline]
    pub const fn start(&self) -> usize {
        self.span.start
    }

    #[inline]
    pub const fn merge(&mut self, chunk: &Chunk) {
        self.span.end = chunk.span.end;
    }
}
