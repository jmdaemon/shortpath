pub mod bash;
// TODO: Implement the powershell shell completions module

use crate::config::Shortpaths;

use std::{
    fs,
    path::PathBuf
};

/** Get the local platform independent shell completions path
  * This results in:
  * Bash        : ./completions/shortpaths.bash
  * Powershell  : ./completions/shortpaths.ps1
  */
pub fn get_shell_completions_path(shell_type: &str) -> String {
    match shell_type {
        "bash" => bash::fmt_export_path(),
        _ => String::from("")
    }
}

/** Get the local platform independent shell completions path
  * This results in:
  * Bash        : /usr/share/bash-completion/completions/shortpaths
  * Powershell  : $profile/shortpaths.ps1
  */

pub fn get_shell_completions_sys_path(shell_type: &str) -> String {
    match shell_type {
        "bash" => bash::fmt_export_sys_path(),
        _ => String::from("")
    }
}

/** Let only users with equal permissions edit
  * the shell completions file */
pub fn set_shell_completions_perms(_shell_type: &str) {
    todo!("Set user completion file perms");
}

/** Generate shell completion files for a given shell type */
pub fn gen_shell_completions(shell_type: &str, spaths: &Shortpaths) -> String {
    match shell_type {
        "bash"          => bash::serialize_bash(&spaths),
        _               => String::from("Not yet implemented"),
    }
}

/** Writes outputs to disk
  * If this function fails, execution stops */
pub fn export(dest: &PathBuf, output: String) {
    fs::write(dest, output).expect("Could not export file");
}
