use crate::config::{App, Shortpaths};
use crate::export as E;
use crate::export::get_exporter;
use crate::shortpaths::{find_matching_path, fold_shortpath};

use std::{
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use log::{debug, error, trace, info, warn};

//pub fn add(alias_name: &str, alias_path: &str, app: &mut App) {
//pub fn add<S: Into<String> + Copy>(alias_name: S, alias_path: &str, app: &mut App) {

//pub fn add<S, P>(alias_name: S, alias_path: P, app: &mut App)
//where
//S: Into<String>,
//P: Into<PathBuf>

pub fn add(alias_name: &str, alias_path: &Path, app: &mut App)
//S: Into<String> + Copy,
//P: AsRef<PathBuf>
//P: Into<PathBuf> + Copy
{
    //let path = PathBuf::from(alias_path.into());
    //app.shortpaths.insert(alias_name.into(), path);
    //let path = alias_path.into();
    //app.shortpaths.insert(alias_name.into(), path);
    app.shortpaths.insert(alias_name.into(), alias_path.into());
    //println!("Saved shortpath {}: {}", alias_name.into(), &path.to_path_buf().display());
    //println!("Saved shortpath {}: {}", alias_name.into(), &alias_path.into().display());
}

pub fn remove(current_name: &str, app: &mut App) {
    let path = app.shortpaths.remove_by_left(current_name).unwrap().1;
    println!("Removed {}: {}", current_name.to_owned(), path.display());
}

pub fn check(app: App) {
    app.shortpaths.into_iter().for_each(|(k, v)| {
        // If the path doesn't exist
        if v.to_str().is_none() || !v.exists() {
            println!("{} shortpath is unreachable: {}", k, v.display());
        }
    });
    println!("Check Complete");
}

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

pub fn export(export_type: &str, output_file: Option<&String>, app: &mut App) {
//pub fn export<S: Into<String>>(export_type: S, output_file: Option<&String>, app: &mut App) {
    //fn new<S: Into<String>>(name: S)
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

    E::export(&dest, output);
    println!("Exported shell completions to {}", &dest.display());
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
