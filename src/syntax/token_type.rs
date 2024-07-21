use eframe::epaint::Color32;
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
}
static DIGIT: Color32 = Color32::from_rgb(0xff, 0, 0);
static KEYWORD1: Color32 = Color32::from_rgb(0x00, 0x66, 0x99);
static KEYWORD2: Color32 = Color32::from_rgb(0x00, 0x99, 0x66);
static KEYWORD3: Color32 = Color32::from_rgb(0x00, 0x99, 0xff);
static KEYWORD4: Color32 = Color32::from_rgb(0x66, 0xcc, 0xff);
static LITERAL1: Color32 = Color32::from_rgb(0xff, 0x00, 0xcc);
static LITERAL2: Color32 = Color32::from_rgb(0xcc, 0x00, 0xcc);
static LITERAL3: Color32 = Color32::from_rgb(0x99, 0x00, 0xcc);
static LITERAL4: Color32 = Color32::from_rgb(0x66, 0x00, 0xcc);
static MARKUP: Color32 = Color32::from_rgb(0x00, 0x00, 0xff);
static OPERATOR: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);

impl Token<'_> {
    pub(crate) fn get_color(&self) -> Color32 {
        match self {
            Token::Bool(_) => LITERAL2,
            Token::BraceOpen => OPERATOR,
            Token::BraceClose => OPERATOR,
            Token::BracketOpen => OPERATOR,
            Token::BracketClose => OPERATOR,
            Token::Colon => OPERATOR,
            Token::Comma => OPERATOR,
            Token::Null => LITERAL2,
            Token::Number(_) => DIGIT,
            Token::String(_) => LITERAL1,
        }
    }
}
