use crate::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
    CONFIG_FILE_PATH,
};
use crate::config::{Config, read_config};
use crate::sp::SP;

use serde::{Serialize, Deserialize};
use log::LevelFilter;
use pretty_env_logger::formatted_timed_builder;
use clap::{arg, ArgAction, Command, ArgMatches};
//use indexmap::IndexMap;
//use serde_with::serde_as;
//use toml;
 
//#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Shortpaths {
    #[serde(rename(serialize = "shortpaths", deserialize = "shortpaths"))]
    //#[serde_as(as = "IndexMap<_, serde_with::toml::TomlString>")]
    pub paths: SP,
}

// Write up a custom serialization for Serde

pub fn setup_config(file: &str) -> Config {
    let mut config = Config::new();
    let cfg_name = file.to_string();
    //let cfg_path = config.format_config_path(&cfg_name);
    //config.add_config(cfg_name, cfg_path.to_str().unwrap());
    config.add_config(cfg_name, CONFIG_FILE_PATH);
    config
}

impl Shortpaths {
    pub fn new() -> Shortpaths {
        let cfg = setup_config(CONFIG_FILE_PATH);
        let toml_conts = read_config(&cfg, CONFIG_FILE_PATH);
        let shortpaths: Shortpaths = toml::from_str(&toml_conts).unwrap();
        shortpaths
    }
}

// CLI

/// Enable logging with `-v --verbose` flags
pub fn toggle_logging(matches: &ArgMatches) {
    let verbose: &bool = matches.get_one("verbose").unwrap();
    if *verbose == true {
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
                arg!(ALIAS_NAME: -n --name <ALIAS_NAME> "New shortpath name"),
                arg!(ALIAS_PATH: -p --path <ALIAS_PATH> "New shortpath path"),
                ])
        );
    cli
}
