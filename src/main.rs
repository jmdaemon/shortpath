use shortpaths::shortpaths::App;
use shortpaths::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
};

use std::path::Path;

use shortpaths::commands::{add,remove,check, autoindex, export,update};

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
    let mut app = App::new();
    info!("Current App Shortpaths:\n{}", toml::to_string_pretty(&app).expect("Could not serialize"));

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let (alias_name, alias_path) = (
                sub_matches.get_one::<String>("ALIAS_NAME").unwrap().to_owned(),
                sub_matches.get_one::<String>("ALIAS_PATH").unwrap(),
                );
            add(&alias_name, &Path::new(&alias_path), &mut app);
            println!("Saved shortpath {}: {}", alias_name, alias_path);
            app.save_to_disk();
        }
        Some(("remove", sub_matches)) => {
            let current_name = sub_matches.get_one::<String>("ALIAS_NAME").unwrap();
            let path = remove(&current_name, &mut app);
            println!("Removed {}: {}", current_name.to_owned(), path.display());
            app.save_to_disk();
        }
        Some(("check", _)) => {
            check(app);
            println!("Check Complete");
        }
        Some(("autoindex", _)) => {
            println!("Updating shortpaths");
            info!("Finding unreachable shortpaths");
            autoindex(&mut app);
            app.save_to_disk();
        }
        Some(("export", sub_matches)) => {
            let (export_type, output_file) = (
                sub_matches.get_one::<String>("EXPORT_TYPE").unwrap(),
                sub_matches.get_one::<String>("OUTPUT_FILE"),
            );
            export(export_type, output_file, &app);
            println!("Exported shell completions to {}", &output_file.unwrap());
            app.save_to_disk();
        }
        Some(("update", sub_matches)) => {
            let (current_name, alias_name, alias_path) = (
                sub_matches.get_one::<String>("CURRENT_NAME").unwrap(),
                sub_matches.get_one::<String>("ALIAS_NAME"),
                sub_matches.get_one::<String>("ALIAS_PATH"),
                );
            update(current_name, alias_name, alias_path, &mut app);
            app.save_to_disk();
        }
        _ => {}
    }
}
