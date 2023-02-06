use crate::{
    consts::PROGRAM_NAME,
    export::Export,
    shortpaths::{SP, sort_shortpaths},
    helpers::expand_tilde,
};

use std::{
    path::{Path, PathBuf},
    fs::write,
};

use log::trace;
use const_format::formatcp;

// Constant Strings
pub const BASH_DEFAULT: &str    = formatcp!("completions/{PROGRAM_NAME}.bash");
pub const BASH_SYSTEM: &str     = formatcp!("/usr/share/bash-completion/completions/{PROGRAM_NAME}");

/* NOTE: Consider generating an actual bash completions
   file instead of just generating the aliases script. */
pub struct BashExporter {
    shortpaths: Option<SP>,
}

pub fn fmt_bash_alias(name: &str, path: &Path) -> String {
    format!("export {}=\"{}\"\n", name, path.display())
}

impl Default for BashExporter {
    fn default() -> Self {
        BashExporter::new(None)
    }
}

impl BashExporter {
    pub fn new(shortpaths: Option<SP>) -> BashExporter {
        BashExporter { shortpaths }
    }
}

impl Export for BashExporter {
    fn get_completions_path(&self) -> String { BASH_DEFAULT.to_owned() }
    fn get_completions_sys_path(&self) -> String { BASH_SYSTEM.to_owned() }
    fn get_completions_user_path(&self) -> String {
        let data_dir = dirs::data_dir();
        format!("{}/bash-completion/completions/{}", data_dir.unwrap().to_str().unwrap(), PROGRAM_NAME)
    }

    fn set_completions_fileperms(&self) -> String {
        todo!("Set user completion file perms");
    }

    fn gen_completions(&self) -> String {
        let mut output = String::from("#!/bin/bash\n\n");
        let cpy = self.shortpaths.clone();
        let shortpaths = sort_shortpaths(cpy.unwrap());

        //if let Some(shortpaths) = shortpaths {
            let serialized: Vec<String> = shortpaths.iter().map(|(name, sp)| {
                println!("shortpaths: {}: {}", &name, sp.path.display());
                println!("shortpaths: {}: {:?}", &name, sp.full_path);
                let path = expand_tilde(&sp.path).unwrap();
                fmt_bash_alias(name, &path)
            }).collect();

            serialized.iter().for_each(|line| output += line);
            trace!("output: {}", output);
        //}
        output
    }

    fn write_completions(&self, dest: PathBuf) -> PathBuf {
        let output = self.gen_completions();
        write(&dest, output).expect("Unable to write to disk");
        dest
    }

    fn set_shortpaths(&mut self, shortpaths: &SP) -> Box<dyn Export> {
        let bexp = BashExporter { shortpaths: Some(sort_shortpaths(shortpaths.to_owned()) ) };
        Box::new(bexp)
    }
}
