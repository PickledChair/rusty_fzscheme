use std::io::{self, Write};

use crate::ast::Node;

pub fn prim_car(args: Vec<Node>) -> Node {
    if let Node::List(items) = args[0].clone() {
        items[0].clone()
    } else {
        Node::Error(format!("car: argument is not pair: {}", args[0].inspect()))
    }
}

pub fn prim_cdr(args: Vec<Node>) -> Node {
    if let Node::List(items) = args[0].clone() {
        let items = items[1..].to_vec();
        if items.len() == 1 {
            items[0].clone()
        } else {
            Node::List(items)
        }
    } else {
        Node::Error(format!("cdr: argument is not pair: {}", args[0].inspect()))
    }
}

pub fn prim_cons(args: Vec<Node>) -> Node {
    let fst = args[0].clone();
    let snd = args[1].clone();
    let mut v = vec![fst];
    if snd.is_pair() {
        if let Node::List(nodes) = snd {
            for node in nodes {
                v.push(node.clone());
            }
        } else {
            unreachable!("snd is pair, so it is also Node::List.");
        }
    } else {
        v.push(snd);
    }
    Node::List(v)
}

pub fn prim_eq(args: Vec<Node>) -> Node {
    let fst = args[0].clone();
    let snd = args[1].clone();
    Node::Bool(fst == snd)
}

pub fn prim_eqv(args: Vec<Node>) -> Node {
    prim_eq(args)
}

pub fn prim_pair(args: Vec<Node>) -> Node {
    Node::Bool(args[0].is_pair())
}

pub fn prim_display(args: Vec<Node>) -> Node {
    let content = if let Node::Str(string) = args[0].clone() {
        string
    } else {
        args[0].inspect()
    };
    print!("{}", content);
    io::stdout().flush().unwrap();
    Node::Undef
}

pub fn prim_newline(_: Vec<Node>) -> Node {
    println!();
    Node::Undef
}

pub fn prim_plus(args: Vec<Node>) -> Node {
    let mut result = 0;
    let mut args = args;
    // 末尾の nil を削除
    args.pop();
    for arg in args {
        match arg {
            Node::Int(num) => result += num,
            Node::Error(_) => return arg,
            _ => {
                return Node::Error(format!(
                    "cannot apply `+` for non-integer object: {}",
                    arg.inspect()
                ))
            }
        }
    }
    Node::Int(result)
}

pub fn prim_times(args: Vec<Node>) -> Node {
    let mut result = 1;
    let mut args = args;
    // 末尾の nil を削除
    args.pop();
    for arg in args {
        match arg {
            Node::Int(num) => result *= num,
            Node::Error(_) => return arg,
            _ => {
                return Node::Error(format!(
                    "cannot apply `*` for non-integer object: {}",
                    arg.inspect()
                ))
            }
        }
    }
    Node::Int(result)
}

pub fn prim_minus(args: Vec<Node>) -> Node {
    let mut args = args;
    // 末尾の nil を削除
    args.pop();
    if args.is_empty() {
        return Node::Error("`-`: arguments is empty".to_string());
    }
    if let Node::Int(num) = args[0].clone() {
        if args.len() == 1 {
            Node::Int(-num)
        } else {
            let mut result = num;
            args.remove(0);
            for arg in args {
                match arg {
                    Node::Int(num) => result -= num,
                    Node::Error(_) => return arg,
                    _ => {
                        return Node::Error(format!(
                            "cannot apply `-` for non-integer object: {}",
                            arg.inspect()
                        ))
                    }
                }
            }
            Node::Int(result)
        }
    } else {
        Node::Error(format!(
            "cannot apply `-` for non-integer object: {}",
            args[0].inspect()
        ))
    }
}

pub fn prim_div(args: Vec<Node>) -> Node {
    if args.len() < 3 {
        return Node::Error(format!(
            "div: shortage of the numbers of arguments {}",
            args.len() - 1
        ));
    }
    let fst = args[0].clone();
    let snd = args[1].clone();
    if let Node::Int(fst_num) = fst {
        if let Node::Int(snd_num) = snd {
            Node::Int(fst_num / snd_num)
        } else {
            Node::Error(format!(
                "`div`: second argument is not integer: {}",
                args[1].inspect()
            ))
        }
    } else {
        Node::Error(format!(
            "`div`: first argument is not integer: {}",
            args[0].inspect()
        ))
    }
}

pub fn prim_modulo(args: Vec<Node>) -> Node {
    if args.len() < 3 {
        return Node::Error(format!(
            "modulo: shortage of the numbers of arguments {}",
            args.len() - 1
        ));
    }
    let fst = args[0].clone();
    let snd = args[1].clone();
    if let Node::Int(fst_num) = fst {
        if let Node::Int(snd_num) = snd {
            Node::Int(fst_num % snd_num)
        } else {
            Node::Error(format!(
                "`modulo`: second argument is not integer: {}",
                args[1].inspect()
            ))
        }
    } else {
        Node::Error(format!(
            "`modulo`: first argument is not integer: {}",
            args[0].inspect()
        ))
    }
}

pub fn prim_ope_equal(args: Vec<Node>) -> Node {
    let mut args = args;
    // 末尾の nil を削除
    args.pop();
    if args.len() < 2 {
        return Node::Error(format!(
            "`=`: shortage of the numbers of arguments {}",
            args.len()
        ));
    }
    let fst = args.remove(0);
    for arg in args {
        match arg {
            Node::Int(_) => {
                if fst == arg {
                    continue;
                } else {
                    return Node::Bool(false);
                }
            }
            Node::Error(_) => return arg,
            _ => {
                return Node::Error(format!(
                    "cannot apply `=` for non-integer object: {}",
                    arg.inspect()
                ))
            }
        }
    }
    Node::Bool(true)
}

pub fn prim_lt(args: Vec<Node>) -> Node {
    let mut args = args;
    // 末尾の nil を削除
    args.pop();
    if args.len() < 2 {
        return Node::Error(format!(
            "`<`: shortage of the numbers of arguments {}",
            args.len()
        ));
    }
    if let Node::Int(fst_num) = args[0].clone() {
        args.remove(0);
        let mut prev_num = fst_num;
        for arg in args {
            match arg {
                Node::Int(num) => {
                    if prev_num < num {
                        prev_num = num;
                        continue;
                    } else {
                        return Node::Bool(false);
                    }
                }
                Node::Error(_) => return arg,
                _ => {
                    return Node::Error(format!(
                        "cannot apply `<` for non-integer object: {}",
                        arg.inspect()
                    ))
                }
            }
        }
        Node::Bool(true)
    } else {
        Node::Error(format!(
            "cannot apply `<` for non-integer object: {}",
            args[0].inspect()
        ))
    }
}

pub fn prim_gt(args: Vec<Node>) -> Node {
    let mut args = args;
    // 末尾の nil を削除
    args.pop();
    if args.len() < 2 {
        return Node::Error(format!(
            "`>`: shortage of the numbers of arguments {}",
            args.len()
        ));
    }
    if let Node::Int(fst_num) = args[0].clone() {
        args.remove(0);
        let mut prev_num = fst_num;
        for arg in args {
            match arg {
                Node::Int(num) => {
                    if prev_num > num {
                        prev_num = num;
                        continue;
                    } else {
                        return Node::Bool(false);
                    }
                }
                Node::Error(_) => return arg,
                _ => {
                    return Node::Error(format!(
                        "cannot apply `>` for non-integer object: {}",
                        arg.inspect()
                    ))
                }
            }
        }
        Node::Bool(true)
    } else {
        Node::Error(format!(
            "cannot apply `>` for non-integer object: {}",
            args[0].inspect()
        ))
    }
}

pub fn prim_le(args: Vec<Node>) -> Node {
    let mut args = args;
    // 末尾の nil を削除
    args.pop();
    if args.len() < 2 {
        return Node::Error(format!(
            "`<=`: shortage of the numbers of arguments {}",
            args.len()
        ));
    }
    if let Node::Int(fst_num) = args[0].clone() {
        args.remove(0);
        let mut prev_num = fst_num;
        for arg in args {
            match arg {
                Node::Int(num) => {
                    if prev_num <= num {
                        prev_num = num;
                        continue;
                    } else {
                        return Node::Bool(false);
                    }
                }
                Node::Error(_) => return arg,
                _ => {
                    return Node::Error(format!(
                        "cannot apply `<=` for non-integer object: {}",
                        arg.inspect()
                    ))
                }
            }
        }
        Node::Bool(true)
    } else {
        Node::Error(format!(
            "cannot apply `<=` for non-integer object: {}",
            args[0].inspect()
        ))
    }
}

pub fn prim_ge(args: Vec<Node>) -> Node {
    let mut args = args;
    // 末尾の nil を削除
    args.pop();
    if args.len() < 2 {
        return Node::Error(format!(
            "`>=`: shortage of the numbers of arguments {}",
            args.len()
        ));
    }
    if let Node::Int(fst_num) = args[0].clone() {
        args.remove(0);
        let mut prev_num = fst_num;
        for arg in args {
            match arg {
                Node::Int(num) => {
                    if prev_num >= num {
                        prev_num = num;
                        continue;
                    } else {
                        return Node::Bool(false);
                    }
                }
                Node::Error(_) => return arg,
                _ => {
                    return Node::Error(format!(
                        "cannot apply `>=` for non-integer object: {}",
                        arg.inspect()
                    ))
                }
            }
        }
        Node::Bool(true)
    } else {
        Node::Error(format!(
            "cannot apply `>=` for non-integer object: {}",
            args[0].inspect()
        ))
    }
}
