use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

use crate::{env::Env, inst::Inst};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Node {
    Bool(bool),
    List(Vec<Node>),
    Int(i64),
    Str(String),
    Ident(String),
    Primitive(String, fn(Vec<Node>) -> Node),
    Closure(LinkedList<Inst>, Rc<RefCell<Env>>),
    Macro(LinkedList<Inst>),
    Error(String),
    Undef,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcTag {
    Primitive,
    Closure,
}

impl Node {
    pub fn is_list(&self) -> bool {
        match self {
            Node::List(nodes) => {
                if let Some(last) = nodes.last() {
                    last.is_null()
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Node::List(nodes) => nodes.is_empty(),
            _ => false,
        }
    }

    pub fn is_pair(&self) -> bool {
        match self {
            Node::List(nodes) => !nodes.is_empty(),
            _ => false,
        }
    }

    pub fn nil() -> Self {
        Node::List(Vec::new())
    }

    pub fn inspect(&self) -> String {
        match self {
            Node::Bool(b) => {
                if *b {
                    String::from("#t")
                } else {
                    String::from("#f")
                }
            }
            Node::Int(int) => int.to_string(),
            Node::Str(string) => format!("{:?}", string),
            Node::Ident(ident) => ident.clone(),
            Node::List(nodes) => {
                if nodes.is_empty() {
                    return "()".to_string();
                }
                let mut s = String::from("(");
                for (i, node) in nodes.iter().enumerate() {
                    if i == nodes.len() - 1 {
                        if node.is_null() {
                            break;
                        } else {
                            s += " .";
                        }
                    }
                    if i != 0 {
                        s += " ";
                    }
                    s += &node.inspect();
                }
                s + ")"
            }
            Node::Primitive(name, _) => format!("#<primitive {}>", name),
            Node::Closure(code, _) => format!("#<closure {:?}>", code),
            Node::Macro(code) => format!("#<macro {:?}", code),
            Node::Error(msg) => format!("Error: {}", msg),
            Node::Undef => "#<undef>".to_string(),
        }
    }
}
