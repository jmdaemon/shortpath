use crate::{
    consts::{PROGRAM_NAME, ORGANIZATION, APPLICATION, QUALIFIER},
    export::{Export, ShellExporter},
};

use std::path::Path;

use directories::ProjectDirs;
use const_format::formatcp;

// Constant Strings
pub const POWERSHELL_DEFAULT: &str  = formatcp!("completions/{PROGRAM_NAME}.ps1");

#[derive(Default)]
pub struct PowershellExporter;

impl ShellExporter for PowershellExporter {
    fn get_completions_sys_path(&self) -> String { self.get_completions_user_path() }
    fn get_completions_user_path(&self) -> String {
        let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap();
        let data_dir = proj_dirs.config_dir();
        format!("{}/completions/powershell/{}.ps1", data_dir.to_path_buf().to_str().unwrap(), PROGRAM_NAME)
    }
}

impl Export for PowershellExporter {
    fn get_completions_path(&self) -> String { POWERSHELL_DEFAULT.to_owned() }

    fn format_alias(&self, name: &str, path: &Path) -> String {
        format!("$Env:{} = \"{}\"\n", name, path.display())
    }
}
