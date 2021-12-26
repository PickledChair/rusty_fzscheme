use std::iter::Peekable;
use std::str::Chars;

use crate::token::Token;

#[derive(Debug)]
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            chars: source.chars().peekable(),
        }
    }

    fn read_escape_char(&mut self) -> Result<char, ()> {
        if let Some(ch) = self.chars.next() {
            match ch {
                't' => Ok('\t'),
                'n' => Ok('\n'),
                'r' => Ok('\r'),
                '\\' => Ok('\\'),
                '"' => Ok('"'),
                _ => Ok(ch),
            }
        } else {
            Err(())
        }
    }

    fn read_string_literal(&mut self) -> Result<String, String> {
        let mut s = String::new();
        while let Some(ch) = self.chars.next() {
            if ch == '"' {
                return Ok(s);
            } else if ch == '\\' {
                if let Ok(esc_ch) = self.read_escape_char() {
                    s.push(esc_ch);
                } else {
                    return Err(s);
                }
            } else {
                s.push(ch);
            }
        }
        Err(s)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.chars.peek() {
            if ch.is_ascii_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if Some(&';') == self.chars.peek() {
            while let Some(ch) = self.chars.peek() {
                if *ch == '\n' {
                    self.chars.next();
                    break;
                } else {
                    self.chars.next();
                }
            }
        }
    }

    fn next_token(&mut self) -> Token {
        while let Some(ch) = self.chars.peek() {
            if ch.is_ascii_whitespace() || *ch == ';' {
                self.skip_whitespace();
                self.skip_comment();
            } else {
                break;
            }
        }

        if let Some(ch) = self.chars.next() {
            match ch {
                '(' => Token::Lparen,
                ')' => Token::Rparen,
                '[' => Token::Lbracket,
                ']' => Token::Rbracket,
                '\'' => Token::Quote,
                '.' => Token::Dot,
                '`' => Token::Quasiquote,
                ',' => {
                    if let Some(ch) = self.chars.peek() {
                        if *ch == '@' {
                            self.chars.next();
                            Token::UnquoteSplicing
                        } else {
                            Token::Unquote
                        }
                    } else {
                        Token::Illegal
                    }
                }
                '#' => {
                    if let Some(ch) = self.chars.next() {
                        match ch {
                            't' => Token::True,
                            'f' => Token::False,
                            _ => Token::Illegal,
                        }
                    } else {
                        Token::Illegal
                    }
                }
                '0' => {
                    if let Some(ch) = self.chars.peek() {
                        if ch.is_ascii_whitespace() || *ch == ')' || *ch == ']' {
                            Token::Integer(0)
                        } else {
                            Token::Illegal
                        }
                    } else {
                        Token::Illegal
                    }
                }
                '1'..='9' => {
                    let mut int_buf = String::new();
                    int_buf.push(ch);
                    while let Some(ch) = self.chars.peek() {
                        if ch.is_ascii_digit() {
                            int_buf.push(*ch);
                            self.chars.next();
                        } else if ch.is_ascii_whitespace() || *ch == ')' || *ch == ']' {
                            break;
                        } else {
                            return Token::Illegal;
                        }
                    }
                    Token::Integer(int_buf.parse().unwrap())
                }
                '"' => match self.read_string_literal() {
                    Ok(string) => Token::Str(string),
                    Err(_) => Token::Illegal,
                },
                ch => {
                    let mut ident_buf = String::new();
                    ident_buf.push(ch);
                    while let Some(ch) = self.chars.peek() {
                        if ch.is_ascii_whitespace() || *ch == ')' || *ch == ']' {
                            break;
                        } else if *ch == '(' || *ch == '[' {
                            return Token::Illegal;
                        } else {
                            ident_buf.push(*ch);
                            self.chars.next();
                        }
                    }
                    Token::Ident(ident_buf)
                }
            }
        } else {
            Token::Eof
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.next_token();
        if tok == Token::Eof {
            None
        } else {
            Some(tok)
        }
    }
}

#[cfg(test)]
mod lexer_test {
    use super::Lexer;
    use crate::token::Token;

    #[test]
    fn lex_sexp_test() {
        let source = r#"
(define (add x y) (+ x y))
(add 42 1)
"#;
        let expected = vec![
            Token::Lparen,
            Token::Ident("define".to_string()),
            Token::Lparen,
            Token::Ident("add".to_string()),
            Token::Ident("x".to_string()),
            Token::Ident("y".to_string()),
            Token::Rparen,
            Token::Lparen,
            Token::Ident("+".to_string()),
            Token::Ident("x".to_string()),
            Token::Ident("y".to_string()),
            Token::Rparen,
            Token::Rparen,
            Token::Lparen,
            Token::Ident("add".to_string()),
            Token::Integer(42),
            Token::Integer(1),
            Token::Rparen,
        ];

        let lex = Lexer::new(source);

        for (tok, expected_tok) in lex.zip(expected) {
            assert_eq!(
                tok, expected_tok,
                "expected: {:?}, got: {:?}",
                expected_tok, tok
            );
        }
    }

    #[test]
    fn lex_bool_test() {
        let source = "(if (condition) #t #f)";
        let expected = vec![
            Token::Lparen,
            Token::Ident("if".to_string()),
            Token::Lparen,
            Token::Ident("condition".to_string()),
            Token::Rparen,
            Token::True,
            Token::False,
            Token::Rparen,
        ];

        let lex = Lexer::new(source);

        for (tok, expected_tok) in lex.zip(expected) {
            assert_eq!(
                tok, expected_tok,
                "expected: {:?}, got: {:?}",
                expected_tok, tok
            );
        }
    }

    #[test]
    fn lex_quote_test() {
        let source = "(define nil '())";
        let expected = vec![
            Token::Lparen,
            Token::Ident("define".to_string()),
            Token::Ident("nil".to_string()),
            Token::Quote,
            Token::Lparen,
            Token::Rparen,
            Token::Rparen,
        ];

        let lex = Lexer::new(source);

        for (tok, expected_tok) in lex.zip(expected) {
            assert_eq!(
                tok, expected_tok,
                "expected: {:?}, got: {:?}",
                expected_tok, tok
            );
        }
    }
}
