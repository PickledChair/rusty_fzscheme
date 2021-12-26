use super::VM;
use crate::{ast::Node, compiler::Compiler, env::init_global_env, lexer::Lexer, parser::Parser};

fn vm_test_template(test_name: &str, source: &str, expected: Node) {
    let lex = Lexer::new(source);
    let mut nodes = Parser::new(lex).parse().unwrap();
    let comp = Compiler::new(nodes.remove(0));
    let mut global_env = init_global_env(None);
    let result = comp.compile(&mut global_env);

    assert!(result.is_ok(), "{:?}: compiling failed.", test_name);

    let mut vm = VM::new(result.unwrap());
    let rtn_value = vm.run(&mut global_env);
    assert_eq!(
        rtn_value, expected,
        "expected: {:?}, got: {:?}",
        expected, rtn_value
    );
}

#[test]
fn vm_integer_test() {
    let source = "1";
    let expected = Node::Int(1);
    vm_test_template("vm_integer_test", source, expected);
}

#[test]
fn vm_quote_test() {
    let source = "(quote a)";
    let expected = Node::Ident("a".to_string());
    vm_test_template("vm_quote_test", source, expected);
}

#[test]
fn vm_if_test() {
    let source0 = "(if #t 'a 'b)";
    let expected0 = Node::Ident("a".to_string());
    vm_test_template("vm_if_test (source0)", source0, expected0);

    let source1 = "(if #f 'a 'b)";
    let expected1 = Node::Ident("b".to_string());
    vm_test_template("vm_if_test (source1)", source1, expected1);
}

#[test]
fn vm_car_test() {
    let source = "(car '(a b c))";
    let expected = Node::Ident("a".to_string());
    vm_test_template("vm_car_test", source, expected);
}

#[test]
fn vm_cdr_test() {
    let source = "(cdr '(a b c))";
    let expected = Node::List(vec![
        Node::Ident("b".to_string()),
        Node::Ident("c".to_string()),
        Node::nil(),
    ]);
    vm_test_template("vm_cdr_test", source, expected);
}

#[test]
fn vm_cons_test() {
    let source = "(cons 'a 'b)";
    let expected = Node::List(vec![
        Node::Ident("a".to_string()),
        Node::Ident("b".to_string()),
    ]);
    vm_test_template("vm_cons_test", source, expected);
}

#[test]
fn vm_eq_test() {
    let source0 = "(eq? 'a 'a)";
    let expected0 = Node::Bool(true);
    vm_test_template("vm_eq_test (source0)", source0, expected0);

    let source1 = "(eq? 'a 'b)";
    let expected1 = Node::Bool(false);
    vm_test_template("vm_eq_test (source1)", source1, expected1);
}

#[test]
fn vm_pair_test() {
    let source0 = "(pair? '(a b c))";
    let expected0 = Node::Bool(true);
    vm_test_template("vm_pair_test (source0)", source0, expected0);

    let source1 = "(pair? 'a)";
    let expected1 = Node::Bool(false);
    vm_test_template("vm_pair_test (source1)", source1, expected1);

    let source2 = "(pair? '(a . b))";
    let expected2 = Node::Bool(true);
    vm_test_template("vm_pair_test (source2)", source2, expected2);

    let source3 = "(pair? '())";
    let expected3 = Node::Bool(false);
    vm_test_template("vm_pair_test (source3)", source3, expected3);
}
