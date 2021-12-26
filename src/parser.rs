use std::iter::Peekable;

use crate::{ast::Node, lexer::Lexer, token::Token};

#[derive(Debug)]
pub struct Parser<'a> {
    lex: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: Lexer<'a>) -> Self {
        Parser {
            lex: lex.peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node>, &'static str> {
        let mut nodes = Vec::new();

        while self.lex.peek().is_some() {
            match self.parse_expr() {
                Ok(expr) => nodes.push(expr),
                Err(msg) => return Err(msg),
            }
        }

        Ok(nodes)
    }

    fn parse_expr(&mut self) -> Result<Node, &'static str> {
        if let Some(tok) = self.lex.next() {
            match tok {
                Token::True => Ok(Node::Bool(true)),
                Token::False => Ok(Node::Bool(false)),
                Token::Integer(int) => Ok(Node::Int(int)),
                Token::Str(string) => Ok(Node::Str(string)),
                Token::Ident(ident) => Ok(Node::Ident(ident)),
                Token::Quote => Ok(Node::List(vec![
                    Node::Ident("quote".to_string()),
                    self.parse_expr()?,
                    Node::nil(),
                ])),
                Token::Quasiquote => {
                    if let Some(next_tok) = self.lex.peek() {
                        if matches!(next_tok, Token::Lparen | Token::Lbracket) {
                            Ok(Node::List(vec![
                                Node::Ident("quasiquote".to_string()),
                                self.parse_expr()?,
                                Node::nil(),
                            ]))
                        } else {
                            Err("quasiquote must exists before a list.")
                        }
                    } else {
                        Err("missing expr next to the quasiquote")
                    }
                }
                Token::Unquote => Ok(Node::List(vec![
                    Node::Ident("unquote".to_string()),
                    self.parse_expr()?,
                    Node::nil(),
                ])),
                Token::UnquoteSplicing => Ok(Node::List(vec![
                    Node::Ident("unquote-splicing".to_string()),
                    self.parse_expr()?,
                    Node::nil(),
                ])),
                Token::Lparen | Token::Rparen => self.parse_list(),
                _ => Err("parsing expr failed."),
            }
        } else {
            unreachable!("don't call parse_expr when the next token don't exists.");
        }
    }

    fn parse_list(&mut self) -> Result<Node, &'static str> {
        let mut nodes = Vec::new();

        loop {
            if let Some(tok) = self.lex.peek() {
                match tok {
                    Token::True
                    | Token::False
                    | Token::Integer(_)
                    | Token::Str(_)
                    | Token::Ident(_)
                    | Token::Quote
                    | Token::Quasiquote
                    | Token::Unquote
                    | Token::UnquoteSplicing
                    | Token::Lparen
                    | Token::Lbracket => {
                        if let Ok(node) = self.parse_expr() {
                            nodes.push(node);
                        } else {
                            return Err("parsing list failed.");
                        }
                    }
                    Token::Rparen | Token::Rbracket => {
                        self.lex.next();
                        break;
                    }
                    Token::Dot => {
                        self.lex.next();
                        if let Ok(node) = self.parse_expr() {
                            nodes.push(node);
                            assert!(matches!(
                                self.lex.peek().unwrap(),
                                Token::Rparen | Token::Rbracket
                            ));
                            self.lex.next();
                            return Ok(Node::List(nodes));
                        } else {
                            return Err("parsing list failed.");
                        }
                    }
                    _ => return Err("parsing list failed."),
                };
            } else {
                return Err("parsing list failed: list is not closed.");
            }
        }
        if !nodes.is_empty() {
            nodes.push(Node::nil());
        }

        Ok(Node::List(nodes))
    }
}

#[cfg(test)]
mod parser_test {
    use super::Parser;
    use crate::{ast::Node, lexer::Lexer};

    #[test]
    fn parse_test() {
        let source = r#"
(define (rect-area w h) (* w h))
(display (rect-area 128 256))
(newline)
"#;

        let expected = vec![
            Node::List(vec![
                Node::Ident("define".to_string()),
                Node::List(vec![
                    Node::Ident("rect-area".to_string()),
                    Node::Ident("w".to_string()),
                    Node::Ident("h".to_string()),
                    Node::nil(),
                ]),
                Node::List(vec![
                    Node::Ident("*".to_string()),
                    Node::Ident("w".to_string()),
                    Node::Ident("h".to_string()),
                    Node::nil(),
                ]),
                Node::nil(),
            ]),
            Node::List(vec![
                Node::Ident("display".to_string()),
                Node::List(vec![
                    Node::Ident("rect-area".to_string()),
                    Node::Int(128),
                    Node::Int(256),
                    Node::nil(),
                ]),
                Node::nil(),
            ]),
            Node::List(vec![Node::Ident("newline".to_string()), Node::nil()]),
        ];

        let lex = Lexer::new(source);
        let mut p = Parser::new(lex);
        let result = p.parse().unwrap();

        assert_eq!(
            result, expected,
            "expected: {:?}, got: {:?}",
            expected, result
        );
    }

    #[test]
    fn parse_quote_test() {
        let source = "(if #t 'a 'b)";
        let expected = vec![Node::List(vec![
            Node::Ident("if".to_string()),
            Node::Bool(true),
            Node::List(vec![
                Node::Ident("quote".to_string()),
                Node::Ident("a".to_string()),
                Node::nil(),
            ]),
            Node::List(vec![
                Node::Ident("quote".to_string()),
                Node::Ident("b".to_string()),
                Node::nil(),
            ]),
            Node::nil(),
        ])];

        let lex = Lexer::new(source);
        let mut p = Parser::new(lex);
        let result = p.parse().unwrap();

        assert_eq!(
            result, expected,
            "expected: {:?}, got: {:?}",
            expected, result
        );
    }
}
