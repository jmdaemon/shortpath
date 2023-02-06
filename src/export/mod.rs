pub mod bash;
pub mod powershell;

use std::{
    path::{PathBuf, Path},
    fs::create_dir_all,
};

use crate::export::bash::BashExporter;
use crate::shortpaths::SP;

/** 
  * Export to multiple applications with a single unified interface
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
    /** Ensure the directory exists at runtime */
    fn prepare_directory(&self, output_file: Option<&String>) -> PathBuf {
        let dest = match output_file {
            Some(path)  => Path::new(path).to_path_buf(),
            None        => PathBuf::from(self.get_completions_path())
        };

        create_dir_all(dest.parent().expect("Could not get parent directory"))
            .expect("Could not create shell completions directory");
        dest
    }

    /** Get the default local platform independent shell completions path */
    fn get_completions_path(&self) -> String;

    /** Get the system shell completions file path */
    fn get_completions_sys_path(&self) -> String;

    /** Make exported completions file read + user executable only */
    fn set_completions_fileperms(&self) -> String;

    fn set_shortpaths(&mut self, shortpaths: &SP) -> Box<dyn Export>;

    /** Generate shell completions.
      * If output_file is not None then the file is generated to output_file */
    fn gen_completions(&self, output_file: Option<&String>) -> String;
}

/** Returns the specific exporter */
pub fn get_exporter(shell_type: &str) -> impl Export {
    match shell_type {
        "bash" => BashExporter::default(),
        _ => BashExporter::default()
    }
}
