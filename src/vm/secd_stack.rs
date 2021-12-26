use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

use crate::{
    ast::{Node, ProcTag},
    env::Env,
    inst::Inst,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StackItem {
    Closure(Node),
    Primitive(Node),
    Other(Node),
}

impl StackItem {
    pub fn new(node: Node, tag: Option<ProcTag>) -> Self {
        if let Some(tag) = tag {
            match tag {
                ProcTag::Closure => StackItem::Closure(node),
                ProcTag::Primitive => StackItem::Primitive(node),
            }
        } else {
            StackItem::Other(node)
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackStack {
    stack: LinkedList<StackItem>,
}

impl StackStack {
    pub fn new() -> Self {
        StackStack {
            stack: LinkedList::new(),
        }
    }

    pub fn push(&mut self, item: StackItem) {
        self.stack.push_front(item);
    }

    pub fn pop(&mut self) -> StackItem {
        self.stack.pop_front().unwrap()
    }
}

pub type EnvStack = Rc<RefCell<Env>>;
pub type CodeStack = LinkedList<Inst>;

#[derive(Debug, Clone)]
pub struct DumpItem {
    pub stack: StackStack,
    pub env: EnvStack,
    pub code: CodeStack,
}

impl DumpItem {
    pub fn new(stack: StackStack, env: EnvStack, code: CodeStack) -> Self {
        DumpItem { stack, env, code }
    }
}

#[derive(Debug, Clone)]
pub struct DumpStack {
    dump: LinkedList<DumpItem>,
}

impl DumpStack {
    pub fn new() -> Self {
        DumpStack {
            dump: LinkedList::new(),
        }
    }

    pub fn push(&mut self, dump: DumpItem) {
        self.dump.push_front(dump);
    }

    pub fn pop(&mut self) -> DumpItem {
        self.dump.pop_front().unwrap()
    }
}
