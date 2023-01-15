use shortpaths::config::{App, Shortpaths};
use shortpaths::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
};
use shortpaths::shortpaths::find_matching_path;

use std::{
    path::PathBuf,
};

//use shortpaths::config::{
    //CONFIG_FILE_PATH,
    //ShortpathsConfig,
    //get_config_path, make_config_dir};

use clap::{arg, ArgAction, Command};
use log::{debug, error, trace, info, warn, LevelFilter};
use pretty_env_logger::formatted_timed_builder;

/// Creates the command line interface
pub fn build_cli() -> Command {
    let cli = Command::new(PROGRAM_NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(PROGRAM_DESCRIPTION)
        .arg(arg!(-v --verbose "Toggle verbose information").action(ArgAction::SetTrue))
        .subcommand(
            Command::new("add")
            .about("Add a shortpath")
            .arg(arg!([ALIAS_NAME]).required(true))
            .arg(arg!([ALIAS_PATH]).required(true)),
            )
        .subcommand(
            Command::new("remove")
            .about("Remove a shortpath")
            .arg(arg!([ALIAS_NAME]).required(true))
            )
        .subcommand(
            Command::new("check")
            .about("Checks all shortpaths")
            )
        .subcommand(
            Command::new("autoindex")
            .about("Fixes all shortpaths.")
            )
        .subcommand(
            Command::new("update")
            .about("Update a shortpath")
            .args(
                &[
                arg!([CURRENT_NAME]).required(true),
                arg!(ALIAS_NAME: -n --name <ALIAS_NAME> "New shortpath name"),
                arg!(ALIAS_PATH: -p --path <ALIAS_PATH> "New shortpath path"),
                ])
        );
    cli
}

// TODO: Make a custom expand function when you check for the existence of a path
// We'll need to deal with nested shortpaths soon in order to provide more resilient paths.
// TODO: Make function to get just the filename part of a path

fn main() {
    let matches = build_cli().get_matches();

    // Enable logging with `-v --verbose` flags
    let verbose: &bool = matches.get_one("verbose").unwrap();
    if *verbose == true {
        formatted_timed_builder().filter_level(LevelFilter::Trace).init();
    }

    // Setup initial configs
    let mut app = App::default();
    info!("Current App Shortpaths:\n{}", toml::to_string_pretty(&app).expect("Could not serialize"));

    // lib.rs: shortpaths
    // 1. Make add, remove, check, update functions

    // lib.rs: export
    // 1. make_shell_completions function to generate bash shell completions

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let (alias_name, alias_path) = (
                sub_matches.get_one::<String>("ALIAS_NAME").unwrap().to_owned(),
                sub_matches.get_one::<String>("ALIAS_PATH").unwrap(),
                );

            let path = PathBuf::from(alias_path);
            println!("Saved shortpath {}: {}", alias_name, alias_path);
            app.shortpaths.insert(alias_name, path);
            app.save_to_disk();
        }
        Some(("remove", sub_matches)) => {
            let current_name = sub_matches.get_one::<String>("ALIAS_NAME").unwrap();
            let path = app.shortpaths.remove(current_name).unwrap();
            println!("Removed {}: {}", current_name.to_owned(), path.display());
            app.save_to_disk();
        }
        Some(("check", _)) => {
            app.shortpaths.into_iter().for_each(|(k, v)| {
                // If the path doesn't exist
                if v.to_str().is_none() || !v.exists() {
                    println!("{} shortpath is unreachable: {}", k, v.display());
                }
            });
            println!("Check Complete");
            // For all shortpaths
            // Check that the path name exist, if it does not, then attempt to find
            // Go through every path in order even if they fail
        }
        Some(("autoindex", _)) => {
            println!("Updating shortpaths");
            info!("Finding unreachable shortpaths");
            let shortpaths: Shortpaths = app.shortpaths
                .into_iter()
                .map(|(alias_name, alias_path)| {
                    if !alias_path.exists() {
                        //find_matching_path(alias_path.as_path());
                        let path = find_matching_path(alias_path.as_path());
                        println!("Updating shortpath {} from {} to {}", alias_name, alias_path.display(), path.display());
                        (alias_name, path)
                    } else {
                        (alias_name, alias_path)
                    }
                }).collect();
            app.shortpaths = shortpaths;
            app.save_to_disk();

            //for (k, v) in app.shortpaths.into_iter() {
                //if v.to_str().is_none() || !v.exists() {
                    //println!("{} shortpath is unreachable: {}", k, v.display());
                    //let path = find_matching_path(&v);
                    ////app.shortpaths.insert(k.clone(), path);
                    //app.shortpaths.insert(k.clone(), path);
                //}
            //}


            //app.shortpaths.into_iter().for_each(|(k, v)| {
                //// If the path doesn't exist
                //if v.to_str().is_none() || !v.exists() {
                    //println!("{} shortpath is unreachable: {}", k, v.display());
                    //let path = find_matching_path(&v);
                    ////app.shortpaths.insert(k.clone(), path);
                    //app.shortpaths.insert(k.clone(), path);
                //}
            //});
            // For all shortpaths
            // Check that the path name exist, if it does not, then attempt to find
            // Go through every path in order even if they fail
        }

        Some(("update", sub_matches)) => {
            let (current_name, alias_name, alias_path) = (
                sub_matches.get_one::<String>("CURRENT_NAME").unwrap(),
                sub_matches.get_one::<String>("ALIAS_NAME"),
                sub_matches.get_one::<String>("ALIAS_PATH"),
                );

            match alias_path {
                Some(new_path) => {
                    let path = PathBuf::from(new_path);
                    app.shortpaths.insert(current_name.to_owned(), path);
                }
                None => {}
            }

            match alias_name {
                Some(new_name) => {
                    let path = app.shortpaths.remove(current_name).unwrap();
                    app.shortpaths.insert(new_name.to_owned(), path);
                }
                None => {}
            }
            app.save_to_disk();
        }
        _ => {}
    }

}
