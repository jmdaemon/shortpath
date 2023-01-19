pub mod bash;
// TODO: Implement the powershell shell completions module

use crate::config::Shortpaths;
use crate::export::bash::BashExporter;

use std::{
    fs,
    path::PathBuf
};

/*
 * Completion Paths
 * Bash        : ./completions/shortpaths.bash
 * Powershell  : ./completions/shortpaths.ps1
 */
pub trait Export {
    fn get_completions_path(&self) -> String;
    fn get_completions_sys_path(&self) -> String;
    fn set_completions_fileperms(&self) -> String;
    fn gen_completions(&self) -> String;
    fn set_shortpaths(&mut self, spaths: &Shortpaths);
}

pub fn get_exporter(shell_type: &str) -> Box<dyn Export> {
    match shell_type {
        "bash" => Box::new(BashExporter::default()),
        _ => Box::new(BashExporter::default())
    }
}

/** Writes outputs to disk
  * If this function fails, execution stops */
pub fn export(dest: &PathBuf, output: String) {
    fs::write(dest, output).expect("Could not export file");
}
