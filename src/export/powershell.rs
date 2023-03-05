use crate::{
    consts::{PROGRAM_NAME, ORGANIZATION, APPLICATION, QUALIFIER},
    export::{Export, ShellExporter},
    shortpaths::{SP, sort_shortpaths},
    env::EnvVars,
};

use std::path::Path;

use directories::ProjectDirs;
use const_format::formatcp;

use super::gen_completions;

// Constant Strings
pub const POWERSHELL_DEFAULT: &str  = formatcp!("completions/{PROGRAM_NAME}.ps1");

#[derive(Default)]
pub struct PowershellExporter {
    shortpaths: Option<SP>,
    env_vars: Option<EnvVars>,
}

impl PowershellExporter {
    pub fn new(shortpaths: Option<SP>, env_vars: Option<EnvVars>) -> PowershellExporter {
        PowershellExporter { shortpaths, env_vars }
    }
}

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

    fn gen_completions(&self) -> String {
        if let Some(shortpaths) = &self.shortpaths {
            let init_fn = || self.init_completions();
            let transpile_fn = |name: &str, path: &Path| self.format_alias(name, path);
            gen_completions(shortpaths.to_owned(), init_fn, transpile_fn)
        } else {
            panic!("shortpaths was None in PowershellExporter");
        }
    }

    fn set_shortpaths(&mut self, shortpaths: &SP) -> Box<dyn Export> {
        let env_vars = self.env_vars.clone();
        let bexp = PowershellExporter { shortpaths: Some(sort_shortpaths(shortpaths.to_owned()) ), env_vars };
        Box::new(bexp)
    }

    fn set_env_vars(&mut self, env_vars: &EnvVars) -> Box<dyn Export> {
        let shortpaths = self.shortpaths.clone();
        let bexp = PowershellExporter { env_vars: Some(env_vars.to_owned()), shortpaths};
        Box::new(bexp)
    }
}
