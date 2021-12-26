use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

use crate::{
    ast::Node,
    env::{Env, GlobalEnv},
    inst::Inst,
    vm::{DumpItem, DumpStack, StackItem, StackStack, VM},
};

pub struct Compiler {
    node: Node,
}

impl Compiler {
    pub fn new(node: Node) -> Self {
        Compiler { node }
    }

    pub fn compile(self, global_env: &mut GlobalEnv) -> Result<LinkedList<Inst>, &'static str> {
        let mut stop_code = LinkedList::new();
        stop_code.push_back(Inst::Stop);
        compile_expr(
            self.node,
            Rc::new(RefCell::new(Env::new())),
            global_env,
            &mut stop_code,
        )
    }
}

fn compile_expr(
    expr: Node,
    env: Rc<RefCell<Env>>,
    global_env: &mut GlobalEnv,
    code: &mut LinkedList<Inst>,
) -> Result<LinkedList<Inst>, &'static str> {
    let mut new_code = LinkedList::new();
    match expr {
        Node::Bool(_) | Node::Int(_) | Node::Str(_) => {
            new_code.push_back(Inst::Ldc(expr));
            new_code.append(code);
            Ok(new_code)
        }
        Node::Ident(_) => {
            if let Some(pos) = env.borrow().location(&expr) {
                new_code.push_back(Inst::Ld(pos.0, pos.1));
                new_code.append(code);
                Ok(new_code)
            } else {
                new_code.push_back(Inst::Ldg(expr));
                new_code.append(code);
                Ok(new_code)
            }
        }
        Node::List(nodes) => {
            if let Some(fst) = nodes.first() {
                match fst {
                    Node::Ident(ident) => {
                        if ident == "quote" {
                            if let Some(snd) = nodes.get(1) {
                                new_code.push_back(Inst::Ldc(snd.clone()));
                                new_code.append(code);
                                return Ok(new_code);
                            } else {
                                return Err("shortage of the args of `quote`.");
                            }
                        } else if ident == "if" {
                            let mut nodes = nodes.clone();
                            // 末尾の nil を削除
                            nodes.pop();
                            let second = nodes.get(1);
                            let third = nodes.get(2);
                            let forth = nodes.get(3);
                            if second.is_none() || third.is_none() {
                                return Err("shortage of the args of `if`.");
                            }
                            let mut join_code = LinkedList::new();
                            join_code.push_back(Inst::Join);
                            let t_clause = compile_expr(
                                third.unwrap().clone(),
                                env.clone(),
                                global_env,
                                &mut join_code,
                            )?;
                            let f_clause = if forth.is_none() {
                                let mut join_code = LinkedList::new();
                                join_code.push_back(Inst::Ldc(Node::Undef));
                                join_code.push_back(Inst::Join);
                                join_code
                            } else {
                                let mut join_code = LinkedList::new();
                                join_code.push_back(Inst::Join);
                                compile_expr(
                                    forth.unwrap().clone(),
                                    env.clone(),
                                    global_env,
                                    &mut join_code,
                                )?
                            };
                            new_code.push_back(Inst::Sel(t_clause, f_clause));
                            new_code.append(code);
                            return compile_expr(
                                second.unwrap().clone(),
                                env,
                                global_env,
                                &mut new_code,
                            );
                        } else if ident == "lambda" {
                            let mut body = nodes.clone();
                            body.remove(0);
                            if body.get(0).is_none() || body.get(1).is_none() {
                                return Err("shortage of the args of `lambda`.");
                            }
                            let args = body.remove(0);

                            let new_env = Rc::new(RefCell::new(Env::new()));
                            new_env.borrow_mut().set_node(args);
                            new_env.borrow_mut().set_next_env(env);

                            let mut rtn_code = LinkedList::new();
                            rtn_code.push_back(Inst::Rtn);
                            let body = compile_body(body, new_env, global_env, &mut rtn_code)?;
                            new_code.push_back(Inst::Ldf(body));
                            new_code.append(code);
                            return Ok(new_code);
                        } else if ident == "define" {
                            let second = nodes.get(1);
                            let third = nodes.get(2);
                            if second.is_none() || third.is_none() {
                                return Err("shortage of the args of `define`.");
                            }
                            let mut second = nodes.get(1).unwrap().clone();
                            let mut third = nodes.get(2).unwrap().clone();

                            match second.clone() {
                                Node::Ident(_) => (),
                                Node::List(mut define_fst_list) => {
                                    // (define (name arg ...) body ...) を
                                    // (define name (lambda (arg ...) body ...)) に解釈し直す
                                    if define_fst_list.get(0).is_none() {
                                        return Err(
                                            "proc name not found in `define` first argument.",
                                        );
                                    }
                                    let proc_name = define_fst_list.remove(0);
                                    second = proc_name;

                                    let mut lambda_node_list = Vec::new();
                                    lambda_node_list.push(Node::Ident("lambda".to_string()));
                                    lambda_node_list.push(Node::List(define_fst_list));
                                    let mut body = nodes.clone();
                                    body.remove(0);
                                    body.remove(0);
                                    lambda_node_list.extend(body);
                                    third = Node::List(lambda_node_list);
                                }
                                _ => {
                                    return Err(
                                        "can accept only symbol or list as first arg of `define`.",
                                    );
                                }
                            }

                            new_code.push_back(Inst::Def(second));
                            new_code.append(code);
                            return compile_expr(third, env, global_env, &mut new_code);
                        } else if ident == "define-macro" {
                            let second = nodes.get(1);
                            let third = nodes.get(2);
                            if second.is_none() || third.is_none() {
                                return Err("shortage of the args of `define-macro`.");
                            }
                            let second = nodes.get(1).unwrap().clone();
                            let third = nodes.get(2).unwrap().clone();

                            match second {
                                Node::Ident(_) => (),
                                _ => {
                                    return Err("can accept only symbol as first arg of `define-macro` currently.");
                                }
                            }

                            new_code.push_back(Inst::Defm(second));
                            new_code.append(code);
                            return compile_expr(third, env, global_env, &mut new_code);
                        } else if ident == "set!" {
                            if nodes.get(1).is_none() || nodes.get(2).is_none() {
                                return Err("shortage of the args of `set!`.");
                            }
                            if let Some((i, j)) = env.borrow().location(&nodes[1]) {
                                new_code.push_back(Inst::Lset(i, j));
                                new_code.append(code);
                                return compile_expr(
                                    nodes[2].clone(),
                                    env.clone(),
                                    global_env,
                                    &mut new_code,
                                );
                            } else {
                                new_code.push_back(Inst::Gset(nodes[1].clone()));
                                new_code.append(code);
                                return compile_expr(
                                    nodes[2].clone(),
                                    env.clone(),
                                    global_env,
                                    &mut new_code,
                                );
                            }
                        } else if let Some(macro_code) = get_macro_code(fst, global_env) {
                            let mut vm_ = VM::new(macro_code.clone());

                            let mut macro_env = Env::new();
                            let mut macro_lvar = nodes.clone();
                            macro_lvar.remove(0);
                            macro_env.set_node(Node::List(macro_lvar));
                            vm_.set_env(Rc::new(RefCell::new(macro_env)));

                            let mut macro_dump = DumpStack::new();
                            let mut dump_code = LinkedList::new();
                            dump_code.push_back(Inst::Stop);
                            macro_dump.push(DumpItem::new(
                                StackStack::new(),
                                Rc::new(RefCell::new(Env::new())),
                                dump_code,
                            ));
                            vm_.set_dump(macro_dump);

                            let macro_result = vm_.run(global_env);
                            return compile_expr(macro_result, env, global_env, code);
                        }
                    }
                    _ => (),
                }

                let length = if nodes.last() == Some(&Node::nil()) {
                    nodes.len() - 2
                } else {
                    nodes.len() - 1
                };
                new_code.push_back(Inst::Args(length));
                let mut app_code = LinkedList::new();
                app_code.push_back(Inst::App);
                app_code.append(code);
                new_code.append(&mut compile_expr(
                    fst.clone(),
                    env.clone(),
                    global_env,
                    &mut app_code,
                )?);
                let mut nodes_clone = nodes.clone();
                // nodes は必ず 1 要素以上持っている
                nodes_clone.remove(0);
                Ok(compile_list(
                    Node::List(nodes_clone),
                    env,
                    global_env,
                    &mut new_code,
                )?)
            } else {
                return Err("attempt to evaluate nil.");
            }
        }
        _ => unreachable!("compiler treat only bool, int, ident and list objects."),
    }
}

fn compile_list(
    expr: Node,
    env: Rc<RefCell<Env>>,
    global_env: &mut GlobalEnv,
    code: &mut LinkedList<Inst>,
) -> Result<LinkedList<Inst>, &'static str> {
    if let Node::List(nodes) = expr {
        if nodes.is_empty() || nodes == vec![Node::nil()] {
            Ok(code.clone())
        } else {
            let mut nodes_clone = nodes.clone();
            nodes_clone.remove(0);
            let mut compiled_list =
                compile_list(Node::List(nodes_clone), env.clone(), global_env, code)?;
            compile_expr(
                nodes.first().unwrap().clone(),
                env,
                global_env,
                &mut compiled_list,
            )
        }
    } else {
        unreachable!("compile_list treat only list object.");
    }
}

fn compile_body(
    body: Vec<Node>,
    env: Rc<RefCell<Env>>,
    global_env: &mut GlobalEnv,
    code: &mut LinkedList<Inst>,
) -> Result<LinkedList<Inst>, &'static str> {
    if body.is_empty() {
        unreachable!("prevent body to be empty by following code");
    }
    let mut body = body.clone();
    let fst = body.remove(0);
    if body.is_empty() || body == vec![Node::nil()] {
        compile_expr(fst, env, global_env, code)
    } else {
        let mut pop_code = LinkedList::new();
        pop_code.push_back(Inst::Pop);
        pop_code.append(&mut compile_body(body, env.clone(), global_env, code)?);
        compile_expr(fst, env, global_env, &mut pop_code)
    }
}

fn get_macro_code(sym: &Node, global_env: &GlobalEnv) -> Option<LinkedList<Inst>> {
    if let Node::Ident(sym) = sym {
        if let Some(stack_item) = global_env.get(sym) {
            if let StackItem::Other(Node::Macro(code)) = stack_item {
                Some(code.clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod compiler_test {
    use super::Compiler;
    use crate::{ast::Node, env::init_global_env, inst::Inst, lexer::Lexer, parser::Parser};
    use std::collections::LinkedList;

    fn compile_test_template(test_name: &str, source: &str, expected: LinkedList<Inst>) {
        let lex = Lexer::new(source);
        let mut p = Parser::new(lex);
        let mut nodes = p.parse().unwrap();
        let comp = Compiler::new(nodes.remove(0));
        let mut global_env = init_global_env(None);
        let result = comp.compile(&mut global_env);

        assert!(
            result.is_ok(),
            "{}: compiling failed. msg: {}",
            test_name,
            result.unwrap_err()
        );
        assert_eq!(
            result.clone().unwrap(),
            expected,
            "expected: {:?}, got: {:?}",
            expected,
            result.unwrap()
        );
    }

    #[test]
    fn compile_integer_test() {
        let source = "1";
        let mut expected = LinkedList::new();
        expected.push_back(Inst::Ldc(Node::Int(1)));
        expected.push_back(Inst::Stop);

        compile_test_template("compile_integer_test", source, expected);
    }

    #[test]
    fn compile_quote_test() {
        let source = "(quote a)";
        let mut expected = LinkedList::new();
        expected.push_back(Inst::Ldc(Node::Ident("a".to_string())));
        expected.push_back(Inst::Stop);

        compile_test_template("compile_quote_test", source, expected);
    }

    #[test]
    fn compile_if_test() {
        let source0 = "(if #t 'a 'b)";

        let mut expected0 = LinkedList::new();
        expected0.push_back(Inst::Ldc(Node::Bool(true)));
        let mut t_clause0 = LinkedList::new();
        t_clause0.push_back(Inst::Ldc(Node::Ident("a".to_string())));
        t_clause0.push_back(Inst::Join);
        let mut f_clause0 = LinkedList::new();
        f_clause0.push_back(Inst::Ldc(Node::Ident("b".to_string())));
        f_clause0.push_back(Inst::Join);
        expected0.push_back(Inst::Sel(t_clause0, f_clause0));
        expected0.push_back(Inst::Stop);

        compile_test_template("compile_if_test (source0)", source0, expected0);

        let source1 = "(if #f 'c)";

        let mut expected1 = LinkedList::new();
        expected1.push_back(Inst::Ldc(Node::Bool(false)));
        let mut t_clause1 = LinkedList::new();
        t_clause1.push_back(Inst::Ldc(Node::Ident("c".to_string())));
        t_clause1.push_back(Inst::Join);
        let mut f_clause1 = LinkedList::new();
        f_clause1.push_back(Inst::Ldc(Node::Undef));
        f_clause1.push_back(Inst::Join);
        expected1.push_back(Inst::Sel(t_clause1, f_clause1));
        expected1.push_back(Inst::Stop);

        compile_test_template("compile_if_test (source1)", source1, expected1);
    }

    #[test]
    fn compile_lambda_test() {
        let source0 = "(lambda (x) x)";
        let mut expected0 = LinkedList::new();
        let mut body_code = LinkedList::new();
        body_code.push_back(Inst::Ld(0, 0));
        body_code.push_back(Inst::Rtn);
        expected0.push_back(Inst::Ldf(body_code));
        expected0.push_back(Inst::Stop);

        compile_test_template("compile_lambda_test (source0)", source0, expected0);

        let source1 = "(lambda () 1 2 3 4 5)";
        let mut expected1 = LinkedList::new();
        let mut body_code = LinkedList::new();
        body_code.push_back(Inst::Ldc(Node::Int(1)));
        body_code.push_back(Inst::Pop);
        body_code.push_back(Inst::Ldc(Node::Int(2)));
        body_code.push_back(Inst::Pop);
        body_code.push_back(Inst::Ldc(Node::Int(3)));
        body_code.push_back(Inst::Pop);
        body_code.push_back(Inst::Ldc(Node::Int(4)));
        body_code.push_back(Inst::Pop);
        body_code.push_back(Inst::Ldc(Node::Int(5)));
        body_code.push_back(Inst::Rtn);
        expected1.push_back(Inst::Ldf(body_code));
        expected1.push_back(Inst::Stop);

        compile_test_template("compile_lambda_test (source1)", source1, expected1);

        let source2 = "(lambda (a . x) (cons a x))";
        let mut expected2 = LinkedList::new();
        let mut body_code = LinkedList::new();
        body_code.push_back(Inst::Ld(0, 0));
        body_code.push_back(Inst::Ld(0, -2));
        body_code.push_back(Inst::Args(2));
        body_code.push_back(Inst::Ldg(Node::Ident("cons".to_string())));
        body_code.push_back(Inst::App);
        body_code.push_back(Inst::Rtn);
        expected2.push_back(Inst::Ldf(body_code));
        expected2.push_back(Inst::Stop);

        compile_test_template("compile_lambda_test (source2)", source2, expected2);
    }

    #[test]
    fn compile_proc_call_test() {
        let source0 = "(car '(a b c))";
        let mut expected0 = LinkedList::new();
        expected0.push_back(Inst::Ldc(Node::List(vec![
            Node::Ident("a".to_string()),
            Node::Ident("b".to_string()),
            Node::Ident("c".to_string()),
            Node::nil(),
        ])));
        expected0.push_back(Inst::Args(1));
        expected0.push_back(Inst::Ldg(Node::Ident("car".to_string())));
        expected0.push_back(Inst::App);
        expected0.push_back(Inst::Stop);

        compile_test_template("compile_proc_call_test (source0)", source0, expected0);

        let source1 = "((lambda (x) x) 'a)";
        let mut expected1 = LinkedList::new();
        expected1.push_back(Inst::Ldc(Node::Ident("a".to_string())));
        expected1.push_back(Inst::Args(1));
        let mut body_code = LinkedList::new();
        body_code.push_back(Inst::Ld(0, 0));
        body_code.push_back(Inst::Rtn);
        expected1.push_back(Inst::Ldf(body_code));
        expected1.push_back(Inst::App);
        expected1.push_back(Inst::Stop);

        compile_test_template("compile_proc_call_test (source1)", source1, expected1);

        let source2 = "((lambda (x y) (cons x y)) 'a 'b)";
        let mut expected2 = LinkedList::new();
        expected2.push_back(Inst::Ldc(Node::Ident("a".to_string())));
        expected2.push_back(Inst::Ldc(Node::Ident("b".to_string())));
        expected2.push_back(Inst::Args(2));
        let mut body_code = LinkedList::new();
        body_code.push_back(Inst::Ld(0, 0));
        body_code.push_back(Inst::Ld(0, 1));
        body_code.push_back(Inst::Args(2));
        body_code.push_back(Inst::Ldg(Node::Ident("cons".to_string())));
        body_code.push_back(Inst::App);
        body_code.push_back(Inst::Rtn);
        expected2.push_back(Inst::Ldf(body_code));
        expected2.push_back(Inst::App);
        expected2.push_back(Inst::Stop);

        compile_test_template("compile_proc_call_test (source2)", source2, expected2);
    }

    #[test]
    fn compile_define_test() {
        let source0 = "(define a 'b)";
        let mut expected0 = LinkedList::new();
        expected0.push_back(Inst::Ldc(Node::Ident("b".to_string())));
        expected0.push_back(Inst::Def(Node::Ident("a".to_string())));
        expected0.push_back(Inst::Stop);

        compile_test_template("compile_define_test (source0)", source0, expected0);

        let source1 = "(define list (lambda x x))";
        let mut expected1 = LinkedList::new();
        let mut body_code = LinkedList::new();
        body_code.push_back(Inst::Ld(0, -1));
        body_code.push_back(Inst::Rtn);
        expected1.push_back(Inst::Ldf(body_code));
        expected1.push_back(Inst::Def(Node::Ident("list".to_string())));
        expected1.push_back(Inst::Stop);

        compile_test_template("compile_define_test (source1)", source1, expected1);

        let source2 = "(define (times a b) (* a b))";
        let mut expected2 = LinkedList::new();
        let mut body_code = LinkedList::new();
        body_code.push_back(Inst::Ld(0, 0));
        body_code.push_back(Inst::Ld(0, 1));
        body_code.push_back(Inst::Args(2));
        body_code.push_back(Inst::Ldg(Node::Ident("*".to_string())));
        body_code.push_back(Inst::App);
        body_code.push_back(Inst::Rtn);
        expected2.push_back(Inst::Ldf(body_code));
        expected2.push_back(Inst::Def(Node::Ident("times".to_string())));
        expected2.push_back(Inst::Stop);

        compile_test_template("compile_define_test (source2)", source2, expected2);
    }
}
