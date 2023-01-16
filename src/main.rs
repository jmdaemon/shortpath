use shortpaths::config::{App, Shortpaths};
use shortpaths::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
};
use shortpaths::shortpaths::find_matching_path;
use shortpaths::export::{
    get_shell_completions_path,
    gen_shell_completions,
    export
};

use std::{
    fs,
    env,
    path::{Path, PathBuf},
    process::exit,
};

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
            Command::new("export")
            .about("Fixes all shortpaths.")
            .args(
                &[
                arg!([EXPORT_TYPE])
                    .required(true)
                    .value_parser(["bash", "powershell"]),
                arg!(OUTPUT_FILE: -o --output <OUTPUT_FILE> "Output to file"),
                ])
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
        }
        Some(("autoindex", _)) => {
            println!("Updating shortpaths");
            info!("Finding unreachable shortpaths");
            let shortpaths: Shortpaths = app.shortpaths
                .into_iter()
                .map(|(alias_name, alias_path)| {
                    if !alias_path.exists() {
                        let path = find_matching_path(alias_path.as_path());
                        println!("Updating shortpath {} from {} to {}", alias_name, alias_path.display(), path.display());
                        (alias_name, path)
                    } else {
                        (alias_name, alias_path)
                    }
                }).collect();
            app.shortpaths = shortpaths;
            app.save_to_disk();
        }
        Some(("export", sub_matches)) => {
            let (export_type, output_file) = (
                sub_matches.get_one::<String>("EXPORT_TYPE").unwrap(),
                sub_matches.get_one::<String>("OUTPUT_FILE"),
            );
            
            // Get export path
            let dest = match output_file {
                Some(path) => {
                    Path::new(path).to_path_buf()
                }
                None => {
                    let p = env::current_dir().unwrap();
                    p.join(get_shell_completions_path(export_type))
                }
            };

            // Make the directories if needed
            fs::create_dir_all(dest.parent().expect("Could not get parent directory"))
                .expect("Could not create shell completions directory");

            // Serialize
            let output = gen_shell_completions(export_type, &app.shortpaths);

            export(&dest, output);
            println!("Exported shell completions to {}", &dest.display());
        }
        Some(("update", sub_matches)) => {
            let (current_name, alias_name, alias_path) = (
                sub_matches.get_one::<String>("CURRENT_NAME").unwrap(),
                sub_matches.get_one::<String>("ALIAS_NAME"),
                sub_matches.get_one::<String>("ALIAS_PATH"),
                );
            
            if alias_name.is_none() && alias_path.is_none() {
                println!("Shortpath name or path must be provided");
                exit(1);
            }
            
            // Change only the name or path of the alias if those are modified and given
            if let Some(new_path) = alias_path {
                let path = PathBuf::from(new_path);
                app.shortpaths.insert(current_name.to_owned(), path);
            } else if let Some(new_name) = alias_name {
                let path = app.shortpaths.remove(current_name).unwrap();
                app.shortpaths.insert(new_name.to_owned(), path);
            } 
            app.save_to_disk();
        }
        _ => {}
    }

}
