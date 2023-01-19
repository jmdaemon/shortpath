use crate::consts::PROGRAM_NAME;
use crate::config::Shortpaths;
use crate::export::Export;

// Idea: Think about creating a proper completions file for bash instead of just
// creating bash aliases

use std::{
    path::{Path, PathBuf},
};
use bimap::BiHashMap;

pub struct BashExporter {
    spaths: Shortpaths,
}

pub fn fmt_bash_alias(name: String, path: &PathBuf) -> String {
    format!("{}=\"{}\"\n", name, path.display())
}
impl BashExporter {
    pub fn new(spaths: Shortpaths) -> BashExporter {
        BashExporter { spaths }
    }

    pub fn default() -> BashExporter {
        BashExporter::new(BiHashMap::new())
    }

    pub fn serialize_bash(&self) -> String {
        let mut output: String = String::new();
        output += "#!/bin/bash\n\n";

        let sp = &self.spaths;
        //for (name, path) in &mut self.spaths.into_iter() {
        for (name, path) in sp.into_iter() {
            //output += &fmt_bash_alias(name.to_string(), &path);
            output += &fmt_bash_alias(name.to_string(), &path);
        }
        output
    }
}

impl Export for BashExporter {
    /// Get the default local platform independent shell completions path 
    fn get_completions_path(&self) -> String {
        format!("completions/{}.bash", PROGRAM_NAME)
    }

    /// Get the system shell completions file path
    fn get_completions_sys_path(&self) -> String {
        format!("/usr/share/bash-completion/completions/{}", PROGRAM_NAME)
    }

    /** Let only users with equal permissions edit
      * the shell completions file */
    fn set_completions_fileperms(&self) -> String {
        todo!("Set user completion file perms");
        //String::from("")
    }

    fn gen_completions(&self) -> String {
        self.serialize_bash()
    }

    fn set_shortpaths(&mut self, spaths: &Shortpaths) {
        self.spaths = spaths.clone();
    }
}
// Unit Tests
#[test]
fn test_serialize_bash() {
    let mut bexp = BashExporter::new(BiHashMap::new());
    bexp.spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());
    //let mut spaths: Shortpaths = BiHashMap::new();
    //spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());
    //let actual = serialize_bash(&spaths);
    let actual = bexp.gen_completions();
    let expect = "#!/bin/bash\n\naaaa=\"/test\"\n";
    assert_eq!(actual, expect);
}
