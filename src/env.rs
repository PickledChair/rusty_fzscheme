use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    ast::{Node, ProcTag},
    compiler::Compiler,
    lexer::Lexer,
    parser::Parser,
    primitive::*,
    vm::{StackItem, VM},
};

fn position_var(sym: &Node, list: &Node) -> Option<isize> {
    match list {
        Node::List(list) => {
            for (i, node) in list.iter().enumerate() {
                if node == sym {
                    if i == list.len() - 1 {
                        return Some(-((i as isize) + 1));
                    } else {
                        return Some(i as isize);
                    }
                }
            }
            None
        }
        Node::Ident(_) => {
            if sym == list {
                Some(-1)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Env {
    node: Node,
    next_env: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            node: Node::nil(),
            next_env: None,
        }
    }

    fn location_helper(&self, expr: &Node, i: usize) -> Option<(usize, isize)> {
        if let Some(j) = position_var(expr, &self.node) {
            Some((i, j))
        } else {
            if let Some(next_env) = &self.next_env {
                next_env.borrow().location_helper(expr, i + 1)
            } else {
                None
            }
        }
    }

    pub fn location(&self, expr: &Node) -> Option<(usize, isize)> {
        self.location_helper(expr, 0)
    }

    fn get_helper(&self, tmp_i: usize, target_i: usize, j: isize) -> Option<Node> {
        if tmp_i == target_i {
            if let Node::List(nodes) = &self.node {
                let mut nodes = nodes.clone();
                if 0 <= j {
                    nodes.get(j as usize).map(|node| node.clone())
                } else {
                    for _ in 0..(-(j + 1)) {
                        nodes.remove(0);
                    }
                    Some(Node::List(nodes))
                }
            } else {
                if j == -1 {
                    Some(self.node.clone())
                } else {
                    None
                }
            }
        } else {
            if let Some(next_env) = &self.next_env {
                next_env.borrow().get_helper(tmp_i + 1, target_i, j)
            } else {
                None
            }
        }
    }

    pub fn get(&self, i: usize, j: isize) -> Option<Node> {
        self.get_helper(0, i, j)
    }

    fn set_helper(&mut self, tmp_i: usize, target_i: usize, j: isize, val: Node) -> Option<()> {
        if tmp_i == target_i {
            if 0 <= j {
                if let Node::List(mut nodes) = self.node.clone() {
                    nodes[j as usize] = val;
                    self.node = Node::List(nodes);
                    Some(())
                } else {
                    unreachable!();
                }
            } else {
                if j == -1 {
                    self.node = val;
                    Some(())
                } else {
                    if let Node::List(mut nodes) = self.node.clone() {
                        nodes.truncate((-(j + 1)) as usize);
                        if let Node::List(val_nodes) = val {
                            for val_node in val_nodes {
                                nodes.push(val_node);
                            }
                        } else {
                            nodes.push(val);
                        }
                        self.node = Node::List(nodes);
                        Some(())
                    } else {
                        unreachable!();
                    }
                }
            }
        } else {
            if let Some(next) = &self.next_env {
                next.borrow_mut().set_helper(tmp_i + 1, target_i, j, val)
            } else {
                None
            }
        }
    }

    pub fn set(&mut self, i: usize, j: isize, val: Node) -> Option<()> {
        self.set_helper(0, i, j, val)
    }

    pub fn set_next_env(&mut self, env: Rc<RefCell<Env>>) {
        self.next_env = Some(env);
    }

    pub fn set_node(&mut self, lvar: Node) {
        self.node = lvar;
    }
}

pub type GlobalEnv = HashMap<String, StackItem>;

macro_rules! register_primitive {
    ($env:expr, $name:expr, $func:expr) => {
        $env.insert(
            $name.to_string(),
            StackItem::new(
                Node::Primitive($name.to_string(), $func),
                Some(ProcTag::Primitive),
            ),
        );
    };
}

pub fn init_global_env(sources: Option<Vec<String>>) -> GlobalEnv {
    let mut env = GlobalEnv::new();

    register_primitive!(env, "car", prim_car);
    register_primitive!(env, "cdr", prim_cdr);
    register_primitive!(env, "cons", prim_cons);
    register_primitive!(env, "eq?", prim_eq);
    register_primitive!(env, "eqv?", prim_eqv);
    register_primitive!(env, "pair?", prim_pair);
    register_primitive!(env, "display", prim_display);
    register_primitive!(env, "newline", prim_newline);
    register_primitive!(env, "+", prim_plus);
    register_primitive!(env, "*", prim_times);
    register_primitive!(env, "-", prim_minus);
    register_primitive!(env, "div", prim_div);
    register_primitive!(env, "modulo", prim_modulo);
    register_primitive!(env, "=", prim_ope_equal);
    register_primitive!(env, "<", prim_lt);
    register_primitive!(env, ">", prim_gt);
    register_primitive!(env, "<=", prim_le);
    register_primitive!(env, ">=", prim_ge);

    compile_lib(&mut env, include_str!("mlib.scm"));

    if let Some(sources) = sources {
        for source in sources {
            compile_lib(&mut env, source);
        }
    }

    env
}

fn compile_lib<T: AsRef<str>>(global_env: &mut GlobalEnv, lib_source: T) {
    let lex = Lexer::new(lib_source.as_ref());
    let mut p = Parser::new(lex);
    let nodes = p.parse().unwrap();
    for node in nodes {
        let comp = Compiler::new(node);
        let code = comp.compile(global_env).unwrap();
        VM::new(code).run(global_env);
    }
}
