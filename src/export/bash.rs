use crate::consts::PROGRAM_NAME;
use crate::config::Shortpaths;

use std::{
    fs,
    path::{Path, PathBuf},
    collections::HashMap,
};

use log::error;

pub fn fmt_bash_alias(name: String, path: &PathBuf) -> String {
    format!("{}=\"{}\"\n", name, path.display())
}

pub fn serialize_bash(spaths: Shortpaths) -> String {
    let mut output: String = String::new();
    output += "#!/bin/bash\n\n";
    for (name, path) in spaths.into_iter() {
        output += &fmt_bash_alias(name, &path);
    }
    output
}

pub fn fmt_export_path() -> String {
    format!("completions/{}.bash", PROGRAM_NAME)
}

pub fn export(dest: &PathBuf, output: String) {
    match fs::write(dest, output) {
        Ok(_) => {}
        Err(e) => {
            error!("Could not export file");
            eprintln!("{e}");
        }
    }
}

//pub fn export_bash(output: &PathBuf) -> String {
//}

///// Exports the shortpaths
//trait Export {
    //pub fn export(&self, output: &PathBuf) -> String;
//}

//pub struct BashExporter {
//}

#[test]
fn test_serialize_bash() {
    let mut spaths: Shortpaths = HashMap::new();
    spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());
    let actual = serialize_bash(spaths);
    let expect = "#!/bin/bash\n\naaaa=\"/test\"\n";
    assert_eq!(actual, expect);
}
