use std::path::PathBuf;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

use rusty_fzscheme::exec::exec;
use rusty_fzscheme::repl::repl;
use rusty_fzscheme::util::get_sources;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new("FZScheme (in Rust)")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("INPUT")
                .help("The input file to exec")
                .default_value("")
                .index(1),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Debug flag (to show VM code)"),
        )
        .arg(
            Arg::with_name("load")
                .short("l")
                .long("load")
                .value_name("FILE")
                .multiple(true)
                .help("Load scheme source files (default: empty path list)"),
        )
        .get_matches();

    let dbg_flag = matches.is_present("debug");

    let load_filepaths = values_t!(matches, "load", PathBuf).unwrap_or(Vec::new());
    let sources = match get_sources(load_filepaths) {
        Ok(sources) => sources,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    let exec_path = value_t!(matches, "INPUT", String).unwrap();
    let exec_flag = !exec_path.is_empty();
    if exec_flag {
        let exec_path = PathBuf::from(exec_path);
        exec(&exec_path, sources);
    } else {
        println!("FZScheme in Rust (version {})\n", VERSION);
        repl(
            dbg_flag,
            if sources.is_empty() {
                None
            } else {
                Some(sources)
            },
        );
    }
}
