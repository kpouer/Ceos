use crate::ceos::syntax::chunk::Chunk;
use crate::ceos::syntax::token_type::Token;
use log::{debug, info};
use logos::Logos;

pub(crate) struct Tokenizer<'a> {
    pub(crate) tokens: Vec<Chunk<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            tokens: Self::tokenize(text),
        }
    }

    fn tokenize(text: &str) -> Vec<Chunk> {
        debug!("start tokenizing");
        let lex = Token::lexer(text);
        let mut chunks = Vec::new();
        for (token, span) in lex.spanned() {
            let chunk = Chunk::new(token.ok(), span, text);
            chunks.push(chunk);
        }

        chunks
    }

    pub(crate) fn merge_tokens(&mut self) {
        let len = self.tokens.len();
        if len < 2 {
            return;
        }

        let mut out = Vec::with_capacity(len);
        out.push(self.tokens.remove(0));
        while !self.tokens.is_empty() {
            push_item(&mut out, self.tokens.remove(0));
        }
        if len != out.len() {
            debug!(
                "Before merging we have {len} chunks, after we have {} tokens",
                out.len()
            );
        }
        self.tokens = out
    }

    fn dump_tokens(&self) {
        info!("start");
        for chunk in &self.tokens {
            info!("token: {:?}", chunk.token);
        }
        info!("end");
    }
}

fn push_item<'a>(chunks: &mut Vec<Chunk<'a>>, item: Chunk<'a>) {
    if !eventually_merge(chunks, &item) {
        chunks.push(item)
    }
}

fn eventually_merge(chunks: &mut Vec<Chunk>, chunk: &Chunk) -> bool {
    let last = chunks.last_mut().unwrap();
    if last.token == chunk.token {
        last.merge(chunk);
        return true;
    }
    false
}
