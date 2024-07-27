use crate::ceos::syntax::chunk::Chunk;
use crate::ceos::syntax::token_type::Token;
use log::debug;
use logos::Logos;

pub(crate) fn tokenize(text: &str) -> Vec<Chunk> {
    debug!("start tokenizing");
    let lex = Token::lexer(text);
    let mut chunks = Vec::new();
    for (token, span) in lex.spanned() {
        let chunk = Chunk::new(token.ok(), span, text);
        chunks.push(chunk);
    }

    chunks
}
