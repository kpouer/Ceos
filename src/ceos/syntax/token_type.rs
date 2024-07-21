use logos::Logos;

#[derive(Debug, Logos)]
#[logos(skip r"[ \t\r\n\f]+")]
pub(crate) enum Token<'source> {
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),
    #[token("{")]
    BraceOpen,
    #[token("}")]
    BraceClose,
    #[token("[")]
    BracketOpen,
    #[token("]")]
    BracketClose,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("null")]
    Null,
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice())]
    Number(&'source str),
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| lex.slice())]
    String(&'source str),
    #[token("info", ignore(case))]
    Info,
    #[token("warning", ignore(case))]
    #[token("warn", ignore(case))]
    Warning,
    #[token("error", ignore(case))]
    Error,
    #[token("fatal", ignore(case))]
    Fatal,
}
