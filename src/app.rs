use crate::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
};

use log::LevelFilter;
use pretty_env_logger::formatted_timed_builder;
use clap::{arg, ArgAction, Command, ArgMatches};
 
// Command Line Interface

/// Enable logging with `-v --verbose` flags
pub fn toggle_logging(matches: &ArgMatches) {
    let verbose: &bool = matches.get_one("verbose").unwrap();
    if *verbose {
        formatted_timed_builder().filter_level(LevelFilter::Trace).init();
    }
}

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
            .arg(arg!([NAME]).required(true))
            .arg(arg!([PATH]).required(true)),
            )
        .subcommand(
            Command::new("remove")
            .about("Remove a shortpath")
            .arg(arg!([NAME]).required(true))
            )
        .subcommand(
            Command::new("check")
            .about("Checks all shortpaths")
            )
        .subcommand(
            Command::new("resolve")
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
                arg!(NAME: -n --name <NAME> "New shortpath name"),
                arg!(PATH: -p --path <PATH> "New shortpath path"),
                ])
        );
    cli
}
