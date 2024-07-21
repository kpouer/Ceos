use crate::syntax::chunk::Chunk;
use crate::syntax::token_type::Token;
use log::{debug, info};
use logos::Logos;

pub(crate) struct Tokenizer<'a> {
    text: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub(crate) fn tokenize(&self) -> Vec<Chunk> {
        debug!("start tokenizing");
        let lex = Token::lexer(self.text);
        let mut chunks = Vec::new();
        for (token, span) in lex.spanned() {
            let chunk = Chunk::new(token.ok(), span, self.text);
            chunks.push(chunk);
        }

        chunks
    }
}
