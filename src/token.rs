#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    Lparen,
    Rparen,
    Lbracket,
    Rbracket,
    True,
    False,
    Ident(String),
    Integer(i64),
    Str(String),
    Quote,
    Dot,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
    Eof,
    Illegal,
}
