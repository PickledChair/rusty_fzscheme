use std::path::PathBuf;

use crate::env::init_global_env;
use crate::util::get_source;

pub fn exec(path_: &PathBuf, sources: Vec<String>) {
    let mut sources = sources;
    if path_.exists() {
        if path_.is_file() {
            match get_source(&path_) {
                Ok(source) => sources.push(source),
                Err(msg) => {
                    eprintln!("{}", msg);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("INPUT is not file: {}", path_.display());
            std::process::exit(1);
        }
    } else {
        eprintln!("INPUT doesn't exist: {}", path_.display());
        std::process::exit(1);
    }
    init_global_env(Some(sources));
}
