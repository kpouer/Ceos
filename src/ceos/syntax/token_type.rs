use logos::Logos;

#[derive(Debug, Logos)]
#[logos(skip r"[ \t\r\n\f]+")]
pub(crate) enum Token {
    #[token("false")]
    #[token("true")]
    Bool,
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
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
    Number,
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#)]
    String,
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
