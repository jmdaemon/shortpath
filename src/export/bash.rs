use crate::{
    consts::PROGRAM_NAME,
    export::{Export, ShellExporter},
    shortpaths::{SP, sort_shortpaths, substitute_env_paths},
    env::EnvVars,
};

use std::{
    path::{Path, PathBuf},
    fs::{write, set_permissions},
    os::unix::prelude::PermissionsExt,
};

use log::{trace, info};
use const_format::formatcp;

use super::gen_completions;

// Constant Strings
pub const BASH_DEFAULT: &str    = formatcp!("completions/{PROGRAM_NAME}.bash");
pub const BASH_SYSTEM: &str     = formatcp!("/usr/share/bash-completion/completions/{PROGRAM_NAME}");

/* NOTE: Consider generating an actual bash completions
   file instead of just generating the aliases script. */
#[derive(Default)]
pub struct BashExporter {
    shortpaths: Option<SP>,
    env_vars: Option<EnvVars>,
}

//pub fn fmt_bash_alias(name: &str, path: &Path) -> String {
    //format!("export {}=\"{}\"\n", name, path.display())
//}

impl BashExporter {
    pub fn new(shortpaths: Option<SP>, env_vars: Option<EnvVars>) -> BashExporter {
        BashExporter { shortpaths, env_vars }
    }
}

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

     /** Generate shell completions */
    fn gen_completions(&self) -> String {
        if let Some(shortpaths) = &self.shortpaths {
            let init_fn = || self.init_completions();
            let transpile_fn = |name: &str, path: &Path| self.format_alias(name, path);
            gen_completions(shortpaths.to_owned(), init_fn, transpile_fn)
        } else {
            panic!("Shortpaths cant be None in gen_completions");
        }
    }


    //fn gen_completions(&self) -> String {
        //if let Some(shortpaths) = self.shortpaths {
            //gen_completions(shortpaths, String::new, fmt_powershell_alias)
        //} else {
            //panic!("shortpaths was None in PowershellExporter");
        //}
        //info!("gen_completions()");
        //let mut output = String::from("#!/bin/bash\n\n");
        //let shortpaths = substitute_env_paths(self.shortpaths.to_owned().unwrap());
        //shortpaths
            //.iter().for_each(|(name, sp)| {
                //trace!("shortpaths: {}: {}", &name, sp.path.display());
                //trace!("shortpaths: {}: {:?}", &name, sp.full_path);
                //output += &fmt_bash_alias(name, &sp.path);
        //});
        //trace!("output: {}", output);
        //output
    //}

    //fn write_completions(&self, dest: &Path) -> PathBuf {
        //let output = self.gen_completions();
        //write(dest, output).expect("Unable to write to disk");
        //self.set_completions_fileperms(dest);
        //dest.to_path_buf()
    //}

    fn set_shortpaths(&mut self, shortpaths: &SP) -> Box<dyn Export> {
        let env_vars = self.env_vars.clone();
        let bexp = BashExporter { shortpaths: Some(sort_shortpaths(shortpaths.to_owned()) ), env_vars };
        Box::new(bexp)
    }

    fn set_env_vars(&mut self, env_vars: &EnvVars) -> Box<dyn Export> {
        let shortpaths = self.shortpaths.clone();
        let bexp = BashExporter { env_vars: Some(env_vars.to_owned()), shortpaths};
        Box::new(bexp)
    }
}
