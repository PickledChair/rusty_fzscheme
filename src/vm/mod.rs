use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    ast::{Node, ProcTag},
    env::{Env, GlobalEnv},
    inst::Inst,
};

pub mod secd_stack;
pub use secd_stack::*;

#[derive(Debug)]
pub struct VM {
    s: StackStack,
    e: EnvStack,
    c: CodeStack,
    d: DumpStack,
}

impl VM {
    pub fn new(code: CodeStack) -> Self {
        VM {
            s: StackStack::new(),
            e: Rc::new(RefCell::new(Env::new())),
            c: code,
            d: DumpStack::new(),
        }
    }

    pub fn set_env(&mut self, env: Rc<RefCell<Env>>) {
        self.e = env;
    }

    pub fn set_dump(&mut self, dump: DumpStack) {
        self.d = dump;
    }

    pub fn run(&mut self, global_env: &mut GlobalEnv) -> Node {
        loop {
            match self.c.pop_front().unwrap() {
                Inst::Ld(i, j) => {
                    let lvar = get_lvar(&self.e, i, j).unwrap();
                    self.s.push(match lvar {
                        Node::Closure(_, _) => StackItem::new(lvar, Some(ProcTag::Closure)),
                        Node::Primitive(_, _) => StackItem::new(lvar, Some(ProcTag::Primitive)),
                        _ => StackItem::new(lvar, None),
                    });
                }
                Inst::Ldc(node) => {
                    self.s.push(StackItem::new(node, None));
                }
                Inst::Ldg(node) => {
                    if let Node::Ident(ident) = node {
                        if let Some(item) = get_gvar(&ident, global_env) {
                            self.s.push(item);
                        } else {
                            return Node::Error(format!(
                                "symbol not found in the global environment: {}",
                                ident
                            ));
                        }
                    } else {
                        unreachable!("opcode `ldg` treat only ident object.");
                    }
                }
                Inst::Ldf(code) => self.s.push(StackItem::new(
                    Node::Closure(code, self.e.clone()),
                    Some(ProcTag::Closure),
                )),
                Inst::Lset(i, j) => {
                    let stack_top = self.s.pop();
                    let node = match stack_top.clone() {
                        StackItem::Closure(node)
                        | StackItem::Primitive(node)
                        | StackItem::Other(node) => node,
                    };
                    set_lvar(&self.e, i, j, node);
                    self.s.push(stack_top);
                }
                Inst::Gset(node) => {
                    if let Node::Ident(ident) = node {
                        let stack_top = self.s.pop();
                        set_gvar(&ident, stack_top.clone(), global_env);
                        self.s.push(stack_top);
                    } else {
                        unreachable!("opcode `ldg` treat only ident object.");
                    }
                }
                Inst::App => {
                    let (node, tag) = match self.s.pop() {
                        StackItem::Closure(clo) => (clo, ProcTag::Closure),
                        StackItem::Primitive(prim) => (prim, ProcTag::Primitive),
                        _ => {
                            unreachable!("apply only closure or primitive object.")
                        }
                    };
                    let lvar = if let StackItem::Other(node) = self.s.pop() {
                        node
                    } else {
                        unreachable!(
                            "the list of args of closure or primitive is only list object."
                        )
                    };
                    if tag == ProcTag::Primitive {
                        let result = apply(node, lvar);
                        if let Node::Error(_) = result {
                            return result;
                        } else {
                            self.s.push(StackItem::new(result, None));
                        }
                    } else {
                        let dump = DumpItem::new(self.s.clone(), self.e.clone(), self.c.clone());
                        self.d.push(dump);
                        self.s = StackStack::new();
                        if let Node::Closure(clo_code, clo_env) = node {
                            let new_env = Rc::new(RefCell::new(Env::new()));
                            new_env.borrow_mut().set_next_env(clo_env);
                            new_env.borrow_mut().set_node(lvar);
                            self.e = new_env;
                            self.c = clo_code;
                        } else {
                            unreachable!("if ProcTag is Closure, node must be closure object.");
                        }
                    }
                }
                Inst::Rtn => {
                    let save = self.d.pop();
                    let mut s_tmp = self.s.clone();
                    let result = s_tmp.pop();
                    if let StackItem::Other(other) = &result {
                        if let Node::Error(_) = other {
                            return other.clone();
                        }
                    }
                    self.s = save.stack;
                    self.s.push(result);
                    self.e = save.env;
                    self.c = save.code;
                }
                Inst::Sel(then_clause, else_clause) => {
                    if let StackItem::Other(Node::Bool(b)) = self.s.pop() {
                        self.d.push(DumpItem::new(
                            StackStack::new(),
                            Rc::new(RefCell::new(Env::new())),
                            self.c.clone(),
                        ));
                        if b {
                            self.c = then_clause;
                        } else {
                            self.c = else_clause;
                        }
                    } else {
                        self.d.push(DumpItem::new(
                            StackStack::new(),
                            Rc::new(RefCell::new(Env::new())),
                            self.c.clone(),
                        ));
                        self.c = then_clause;
                    }
                }
                Inst::Join => {
                    self.c = self.d.pop().code;
                }
                Inst::Pop => {
                    self.s.pop();
                }
                Inst::Args(num) => {
                    let mut v = Vec::new();
                    for _ in 0..num {
                        match self.s.pop() {
                            StackItem::Other(node) => v.insert(0, node),
                            StackItem::Primitive(node) => v.insert(0, node),
                            StackItem::Closure(node) => v.insert(0, node),
                        }
                    }
                    v.push(Node::nil());
                    self.s.push(StackItem::new(Node::List(v), None));
                }
                Inst::Def(node) => {
                    if let Node::Ident(ident) = node.clone() {
                        global_env.insert(ident, self.s.pop());
                        self.s.push(StackItem::new(node, None));
                    } else {
                        unreachable!("opcode `def` treat only ident object.");
                    }
                }
                Inst::Defm(node) => {
                    if let Node::Ident(ident) = node.clone() {
                        if let StackItem::Closure(Node::Closure(code, _)) = self.s.pop() {
                            global_env.insert(ident, StackItem::new(Node::Macro(code), None));
                            self.s.push(StackItem::new(node, None));
                        } else {
                            panic!();
                        }
                    } else {
                        unreachable!("opcode `defm` treat only ident object.");
                    }
                }
                Inst::Stop => match self.s.pop() {
                    StackItem::Primitive(node) => return node,
                    StackItem::Closure(node) => return node,
                    StackItem::Other(node) => return node,
                },
                // _ => panic!("unimplemented opcode."),
            }
        }
    }
}

fn get_lvar(env: &EnvStack, i: usize, j: isize) -> Option<Node> {
    env.borrow().get(i, j)
}

fn get_gvar(sym: &str, global_env: &GlobalEnv) -> Option<StackItem> {
    global_env.get(sym).map(|item| item.clone())
}

fn set_lvar(env: &EnvStack, i: usize, j: isize, val: Node) -> Option<()> {
    env.borrow_mut().set(i, j, val)
}

fn set_gvar(sym: &str, val: StackItem, global_env: &mut GlobalEnv) -> Option<()> {
    global_env.insert(sym.to_string(), val).map(|_| ())
}

fn apply(primitive: Node, args: Node) -> Node {
    if let Node::Primitive(_, proc) = primitive {
        if let Node::List(args) = args {
            proc(args)
        } else {
            unreachable!("the arg `args` must be Node::List.");
        }
    } else {
        unreachable!("the arg `primitive` must be Node::Primitive.");
    }
}

#[cfg(test)]
mod vm_test;
