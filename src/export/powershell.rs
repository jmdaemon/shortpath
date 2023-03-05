use crate::{
    consts::{PROGRAM_NAME, ORGANIZATION, APPLICATION, QUALIFIER},
    export::Export,
    shortpaths::{SP, sort_shortpaths, substitute_env_paths}, env::EnvVars,
};

use std::{
    path::{Path, PathBuf},
    fs::{write, set_permissions},
    os::unix::prelude::PermissionsExt,
};

use directories::ProjectDirs;
use log::{trace, info};
use const_format::formatcp;

// Constant Strings
pub const POWERSHELL_DEFAULT: &str  = formatcp!("completions/{PROGRAM_NAME}.ps1");

pub struct PowershellExporter {
    shortpaths: Option<SP>,
    env_vars: Option<EnvVars>,
}

pub fn fmt_powershell_alias(name: &str, path: &Path) -> String {
    format!("$Env:{} = \"{}\"\n", name, path.display())
}

impl Default for PowershellExporter {
    fn default() -> Self {
        PowershellExporter::new(None, None)
    }
}

impl PowershellExporter {
    pub fn new(shortpaths: Option<SP>, env_vars: Option<EnvVars>) -> PowershellExporter {
        PowershellExporter { shortpaths, env_vars }
    }
}

impl Export for PowershellExporter {
    fn get_completions_path(&self) -> String { POWERSHELL_DEFAULT.to_owned() }
    fn get_completions_sys_path(&self) -> String { self.get_completions_user_path() }
    fn get_completions_user_path(&self) -> String {
        let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap();
        let data_dir = proj_dirs.config_dir();
        format!("{}/completions/powershell/{}.ps1", data_dir.to_path_buf().to_str().unwrap(), PROGRAM_NAME)
    }

    fn set_completions_fileperms(&self, dest: &Path) {
        let mut perms = dest.metadata().unwrap().permissions();
        perms.set_mode(0o744);
        set_permissions(dest, perms).unwrap_or_else(|_| panic!("Could not set permissions for {}", dest.display()));
    }

    fn gen_completions(&self) -> String {
        info!("gen_completions()");
        let mut output = String::new();
        let shortpaths = substitute_env_paths(self.shortpaths.to_owned().unwrap());
        shortpaths
            .iter().for_each(|(name, sp)| {
                trace!("shortpaths: {}: {}", &name, sp.path.display());
                trace!("shortpaths: {}: {:?}", &name, sp.full_path);
                output += &fmt_powershell_alias(name, &sp.path);
        });
        trace!("output: {}", output);
        output
    }

    fn write_completions(&self, dest: &Path) -> PathBuf {
        let output = self.gen_completions();
        write(dest, output).expect("Unable to write to disk");
        self.set_completions_fileperms(dest);
        dest.to_path_buf()
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
