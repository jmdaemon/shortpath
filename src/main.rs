use shortpaths::shortpaths::App;
use shortpaths::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
};

use std::{
    path::Path,
    process::exit,
} ;

use clap::{arg, ArgAction, Command, ArgMatches};
use log::{info, LevelFilter};
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

// Enable logging with `-v --verbose` flags
pub fn toggle_logging(matches: &ArgMatches) {
    let verbose: &bool = matches.get_one("verbose").unwrap();
    if *verbose == true {
        formatted_timed_builder().filter_level(LevelFilter::Trace).init();
    }
}

fn main() {
    let matches = build_cli().get_matches();
    toggle_logging(&matches);

    // Setup initial configs
    let mut app = App::new();
    info!("Current App Shortpaths:\n{}", toml::to_string_pretty(&app).expect("Could not serialize"));

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let (alias_name, alias_path) = (
                sub_matches.get_one::<String>("ALIAS_NAME").unwrap().to_owned(),
                sub_matches.get_one::<String>("ALIAS_PATH").unwrap(),
                );
            app.add(&alias_name, &Path::new(&alias_path));
            println!("Saved shortpath {}: {}", alias_name, alias_path);
        }
        Some(("remove", sub_matches)) => {
            let current_name = sub_matches.get_one::<String>("ALIAS_NAME").unwrap();
            let path = app.remove(&current_name);
            println!("Removed {}: {}", current_name.to_owned(), path.display());
        }
        Some(("check", _)) => {
            let unreachable = app.check();
            unreachable.iter().for_each(|(alias_name, alias_path)|
                println!("{} shortpath is unreachable: {}", alias_name, alias_path.display()));
            println!("Check Complete");
        }
        Some(("autoindex", _)) => {
            println!("Updating shortpaths");
            info!("Finding unreachable shortpaths");

            let on_update = |alias_name: &String, updated_path: &Path, alias_path: &Path| {
                let is_changed = |p1: &Path, p2: &Path| {p1 != p2};
                if is_changed(updated_path, alias_path) {
                    println!("Updating shortpath {} from {} to {}", alias_name, alias_path.display(), updated_path.display());
                } else {
                    println!("Keeping shortpath {}: {}", alias_name, alias_path.display());
                }
            };
            app.autoindex(Some(on_update));
        }
        Some(("export", sub_matches)) => {
            let (export_type, output_file) = (
                sub_matches.get_one::<String>("EXPORT_TYPE").unwrap(),
                sub_matches.get_one::<String>("OUTPUT_FILE"),
            );
            let dest = app.export(export_type, output_file);
            println!("Exported shell completions to {}", dest);
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
            app.update(current_name, alias_name, alias_path);
        }
        _ => {}
    }
    app.save_to_disk();
}
