use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn get_sources(paths: Vec<PathBuf>) -> Result<Vec<String>, String> {
    let mut v = Vec::new();
    for path_ in paths {
        v.push(get_source(&path_)?);
    }
    Ok(v)
}

pub fn get_source(path_: &PathBuf) -> Result<String, String> {
    if let Ok(mut file) = File::open(&path_) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_err() {
            return Err(format!("couldn't read file: {}", &path_.display()));
        }
        Ok(content)
    } else {
        Err(format!("couldn't open file: {}", &path_.display()))
    }
}
