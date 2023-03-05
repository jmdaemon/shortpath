pub mod bash;
pub mod powershell;

use std::{
    path::{PathBuf, Path},
    fs::{create_dir_all, write, set_permissions},
    os::unix::prelude::PermissionsExt,
};

use crate::{
    app::ExportType,
    export::{bash::BashExporter, powershell::PowershellExporter},
    shortpaths::{SP, substitute_env_paths}
};

use log::{trace, info};

// General purpose functions
/** Make exported completions file rwx by the current only */
fn set_completions_fileperms(dest: &Path) {
    let mut perms = dest.metadata().unwrap().permissions();
    perms.set_mode(0o744);
    set_permissions(dest, perms).unwrap_or_else(|_| panic!("Could not set permissions for {}", dest.display()));
}

/** Write shell completions to disk */
fn write_completions(dest: &Path, output: &str) -> PathBuf {
    write(dest, output).expect("Unable to write to disk");
    set_completions_fileperms(dest);
    dest.to_path_buf()
}

fn gen_completions(shortpaths: SP, init_fn: impl Fn() -> String, transpile_fn: impl Fn(&str, &Path) -> String) -> String {
    info!("gen_completions()");
    let mut output = init_fn();
    let shortpaths = substitute_env_paths(shortpaths);
    shortpaths
        .iter().for_each(|(name, sp)| {
            trace!("shortpaths: {}: {}", &name, sp.path.display());
            trace!("shortpaths: {}: {:?}", &name, sp.full_path);
            output += &transpile_fn(name, &sp.path);
    });
    trace!("output: {}", output);
    output
}

pub trait ShellExporter {
    /** Get the user shell completions file path */
    fn get_completions_user_path(&self) -> String;
    /** Get the system shell completions file path */
    fn get_completions_sys_path(&self) -> String;
}

/** 
  * Export to multiple applications with a single unified interface
  *
  * NOTE:
  * The interface is very shell-related as of right now. When more formats
  * are available, then this trait could be made more abstract.
  *
  * Export Paths:
  * Bash:
  *     Default : ./completions/shortpaths.bash
  *     System  :
  *     User    : 
  *
  * Powershell
  *     Default  : ./completions/shortpaths.ps1
  *     System   : ./completions/shortpaths.ps1
  *     User     :
  */
pub trait Export {
    /** Get the default local platform independent shell completions path */
    fn get_completions_path(&self) -> String;

    /** Ensure the directory exists at runtime */
    fn prepare_directory(&self, output_file: Option<PathBuf>) -> PathBuf {
        let dest = match output_file {
            Some(path)  => path,
            None        => PathBuf::from(self.get_completions_path())
        };

        create_dir_all(dest.parent().expect("Could not get parent directory"))
            .expect("Could not create shell completions directory");
        dest
    }

    fn set_completions_fileperms(&self, dest: &Path) {
        set_completions_fileperms(dest)
    }

    fn format_alias(&self, name: &str, path: &Path) -> String;

    fn init_completions(&self) -> String {
        String::new()
    }

    /** Generate shell completions */
    fn gen_completions(&self, shortpaths: SP) -> String {
        let init_fn = || self.init_completions();
        let transpile_fn = |name: &str, path: &Path| self.format_alias(name, path);
        gen_completions(shortpaths, init_fn, transpile_fn)
    }

    fn write_completions(&self, dest: &Path, shortpaths: SP) -> PathBuf {
        let output = self.gen_completions(shortpaths);
        write_completions(dest, &output)
    }
}

/** Returns the specific exporter */
pub fn get_exporter(export_type: ExportType) -> Box<dyn Export> {
    match export_type {
        ExportType::Bash => Box::<BashExporter>::default(),
        ExportType::Powershell => Box::<PowershellExporter>::default(),
    }
}
