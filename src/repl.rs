use std::io::{self, Write};

use crate::{compiler::Compiler, env::init_global_env, lexer::Lexer, parser::Parser, vm::VM};

pub fn repl(debug: bool, sources: Option<Vec<String>>) {
    let mut global_env = init_global_env(sources);

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        if line.contains("(quit)") || line.contains("(exit)") {
            break;
        }

        let lex = Lexer::new(&line);
        let mut p = Parser::new(lex);
        match p.parse() {
            Ok(nodes) => {
                for node in nodes {
                    let comp = Compiler::new(node);
                    match comp.compile(&mut global_env) {
                        Ok(code) => {
                            if debug {
                                println!("VM code:\n\n{:?}\n", code);
                            }
                            let result = VM::new(code).run(&mut global_env);
                            print!("==> ");
                            io::stdout().flush().unwrap();
                            println!("{}", result.inspect());
                        }
                        Err(msg) => println!("compile error: {}", msg),
                    }
                }
            }
            Err(msg) => println!("parse error: {}", msg),
        }
    }
}
