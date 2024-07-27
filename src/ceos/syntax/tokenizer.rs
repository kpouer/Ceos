use crate::ceos::syntax::chunk::Chunk;
use crate::ceos::syntax::token_type::Token;
use log::{debug, info};
use logos::Logos;

pub(crate) fn tokenize(text: &str) -> Vec<Chunk> {
    debug!("start tokenizing");
    let lex = Token::lexer(text);
    let mut chunks = Vec::new();
    for (token, span) in lex.spanned() {
        let chunk = Chunk::new(token.ok(), span, text);
        chunks.push(chunk);
    }

    merge_tokens(chunks)
}

fn merge_tokens(mut chunks: Vec<Chunk>) -> Vec<Chunk> {
    let len = chunks.len();
    if len < 2 {
        return chunks;
    }

    let mut out = Vec::with_capacity(len);
    out.push(chunks.remove(0));
    while !chunks.is_empty() {
        push_item(&mut out, chunks.remove(0));
    }
    if len != out.len() {
        debug!("Before merging we have {len} chunks, after we have {} tokens", out.len());
    }
    out
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

fn dump_tokens(chunks: &Vec<Chunk>) {
    info!("start");
    for chunk in chunks {
        info!("token: {:?}", chunk.token);
    }
    info!("end");
}