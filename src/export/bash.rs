use crate::consts::PROGRAM_NAME;
use crate::config::Shortpaths;

// Idea: Think about creating a proper completions file for bash instead of just
// creating bash aliases

use std::{
    path::{Path, PathBuf},
    collections::HashMap,
};

pub fn fmt_bash_alias(name: String, path: &PathBuf) -> String {
    format!("{}=\"{}\"\n", name, path.display())
}

pub fn serialize_bash(spaths: &Shortpaths) -> String {
    let mut output: String = String::new();
    output += "#!/bin/bash\n\n";
    for (name, path) in spaths.into_iter() {
        output += &fmt_bash_alias(name.to_string(), &path);
    }
    output
}

pub fn fmt_export_path() -> String {
    format!("completions/{}.bash", PROGRAM_NAME)
}

// Unit Tests
#[test]
fn test_serialize_bash() {
    let mut spaths: Shortpaths = HashMap::new();
    spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());
    let actual = serialize_bash(&spaths);
    let expect = "#!/bin/bash\n\naaaa=\"/test\"\n";
    assert_eq!(actual, expect);
}
