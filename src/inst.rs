use std::collections::LinkedList;

use crate::ast::Node;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Inst {
    Ld(usize, isize),
    Ldc(Node),
    Ldg(Node),
    Ldf(LinkedList<Inst>),
    Lset(usize, isize),
    Gset(Node),
    Args(usize),
    App,
    Rtn,
    Sel(LinkedList<Inst>, LinkedList<Inst>),
    Join,
    Pop,
    Def(Node),
    Defm(Node),
    Stop,
}
