use crate::shortpaths::{App, Shortpaths};
use crate::export::get_exporter;
use crate::shortpaths::{find_matching_path, fold_shortpath};

use std::{
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use log::info;
use bimap::Overwritten;

//pub fn add<S, P>(alias_name: S, alias_path: P, app: &mut App)
//where
//S: Into<String>,
//P: Into<PathBuf>

/// Adds a new shortpath to the list of shortpaths
pub fn add(alias_name: &str, alias_path: &Path, app: &mut App) -> Overwritten<String, PathBuf> {
    app.shortpaths.insert(alias_name.into(), alias_path.into())
}

/// Remove a shortpath
pub fn remove(current_name: &str, app: &mut App) -> PathBuf {
    app.shortpaths.remove_by_left(current_name).unwrap().1
}

/// Checks if all the paths are available
pub fn check(app: App) {
    app.shortpaths.into_iter().for_each(|(k, v)| {
        // If the path doesn't exist
        if v.to_str().is_none() || !v.exists() {
            println!("{} shortpath is unreachable: {}", k, v.display());
        }
    });
}

/** Find and fix all unreachable paths
  * Also expands/folds nested or aliased shortpaths */
pub fn autoindex(app: &mut App) {
    info!("Finding unreachable shortpaths");
    let spaths = app.shortpaths.clone();
    let sp = app.shortpaths.clone();
    let shortpaths: Shortpaths = sp
        .into_iter()
        .map(|(alias_name, alias_path)| {
            if !alias_path.exists() {
                let path = find_matching_path(alias_path.as_path(), &spaths);
                if path != alias_path.as_path() {
                    println!("Updating shortpath {} from {} to {}", alias_name, alias_path.display(), path.display());
                } else {
                    println!("Keeping shortpath {}: {}", alias_name, alias_path.display());
                }
                (alias_name, path)
            } else {
                (alias_name, fold_shortpath(&alias_path.to_path_buf(), &spaths))
            }
        }).collect();
    app.shortpaths = shortpaths;
}

/** Serialize shortpaths to other formats for use in other applications */
pub fn export(export_type: &str, output_file: Option<&String>, app: &App) {
    let mut exp = get_exporter(export_type.into());
    exp.set_shortpaths(&app.shortpaths);
    
    // Get export path
    let dest = match output_file {
        Some(path)  => Path::new(path).to_path_buf(),
        None        => PathBuf::from(exp.get_completions_path())
    };

    // Make the directories if needed
    fs::create_dir_all(dest.parent().expect("Could not get parent directory"))
        .expect("Could not create shell completions directory");

    // Serialize
    let output = exp.gen_completions();
    fs::write(&dest, &output).expect("Unable to write to disk");
}

pub fn update(current_name: &str, alias_name: Option<&String>, alias_path: Option<&String>, app: &mut App) {
    if alias_name.is_none() && alias_path.is_none() {
        println!("Shortpath name or path must be provided");
        exit(1);
    }
    
    // Change only the name or path of the alias if those are modified and given
    if let Some(new_path) = alias_path {
        let path = PathBuf::from(new_path);
        app.shortpaths.insert(current_name.to_owned(), path);
    } else if let Some(new_name) = alias_name {
        let path = app.shortpaths.remove_by_left(current_name).unwrap().1;
        app.shortpaths.insert(new_name.to_owned(), path);
    } 
}
