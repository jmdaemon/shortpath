pub mod bash;
pub mod powershell;

use crate::shortpaths::Shortpaths;
use crate::export::bash::BashExporter;

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
