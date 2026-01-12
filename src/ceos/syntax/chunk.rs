use crate::ceos::syntax::token::Token;
use logos::Span;

#[derive(Debug)]
pub(crate) struct Chunk<'a> {
    pub(crate) token: Option<Token>,
    pub(crate) span: Span,
    text: &'a str,
}

impl<'a> Chunk<'a> {
    #[inline]
    pub(crate) const fn new(token: Option<Token>, span: Span, text: &'a str) -> Self {
        Self { token, span, text }
    }

    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        &self.text[self.span.start..self.span.end]
    }

    #[inline]
    pub(crate) const fn start(&self) -> usize {
        self.span.start
    }

    #[inline]
    pub(crate) const fn merge(&mut self, chunk: &Chunk) {
        self.span.end = chunk.span.end;
    }
}
