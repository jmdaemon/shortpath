use crate::{
    consts::PROGRAM_NAME,
    export::{Export, ShellExporter},
};

use std::path::Path;

use const_format::formatcp;

// Constant Strings
pub const BASH_DEFAULT: &str    = formatcp!("completions/{PROGRAM_NAME}.bash");
pub const BASH_SYSTEM: &str     = formatcp!("/usr/share/bash-completion/completions/{PROGRAM_NAME}");

/* NOTE: Consider generating an actual bash completions
   file instead of just generating the aliases script. */
#[derive(Default)]
pub struct BashExporter;
impl ShellExporter for BashExporter {
    fn get_completions_sys_path(&self) -> String { BASH_SYSTEM.to_owned() }
    fn get_completions_user_path(&self) -> String {
        let data_dir = dirs::data_dir();
        format!("{}/bash-completion/completions/{}", data_dir.unwrap().to_str().unwrap(), PROGRAM_NAME)
    }
}

impl Export for BashExporter {
    fn get_completions_path(&self) -> String { BASH_DEFAULT.to_owned() }

    fn format_alias(&self, name: &str, path: &Path) -> String {
        format!("export {}=\"{}\"\n", name, path.display())
    }

    fn init_completions(&self) -> String {
        String::from("#!/bin/bash\n\n")
    }
}
