pub mod bash;
// TODO: Be sure to implement the powershell shell completions module

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

// TODO: Make function to get the universally installed shell completions path
// TODO: Make function to set completion files to correct user permissions

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
    //match fs::write(dest, output) {
        //Ok(_) => {}
        //Err(e) => {
            //error!("Could not export file");
            //eprintln!("{e}");
        //}
    //}
}
