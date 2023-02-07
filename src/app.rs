use crate::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
    CONFIG_FILE_PATH,
};
use crate::config::{Config, read_config, write_config};
use crate::shortpaths::{SP, populate_expanded_paths, sort_shortpaths};
use crate::helpers::{expand_tilde, tab_align, find_longest_keyname};

use serde::{Serialize, Deserialize};
use log::{LevelFilter, trace};
use pretty_env_logger::formatted_timed_builder;
use clap::{arg, ArgAction, Command, ArgMatches};
 
#[derive(Serialize, Deserialize, Debug)]
pub struct Shortpaths {
    pub shortpaths: SP,
    #[serde(skip)]
    pub cfg: Config,
}

pub fn setup_config(file: &str) -> Config {
    let mut config = Config::new();
    config.add_config(file.to_string(), CONFIG_FILE_PATH);
    config
}

impl Default for Shortpaths {
    fn default() -> Self {
        let cfg = setup_config(CONFIG_FILE_PATH);
        let toml_conts = read_config(&cfg, CONFIG_FILE_PATH);
        let mut sp: Shortpaths = toml::from_str(&toml_conts).unwrap();

        let shortpaths = populate_expanded_paths(&mut sp.shortpaths);
        Shortpaths { shortpaths, cfg }
    }
}

impl Shortpaths {
    pub fn new() -> Shortpaths {
        Default::default()
    }

    pub fn to_disk(&mut self) {
        let shortpaths: SP = self.shortpaths.to_owned().into_iter().map(|(name, mut sp)| {
            let path = expand_tilde(&sp.path).unwrap();
            sp.full_path = Some(path);
            (name, sp)
        }).collect();

        let shortpaths = sort_shortpaths(shortpaths);

        let width = find_longest_keyname(shortpaths.clone()).len();
        self.shortpaths = shortpaths;

        let conts = toml::to_string_pretty(&self).expect("Could not serialize shortpaths");
        let delim = " = ";
        
        let fileconts: Vec<String> = conts.split('\n').map(|line| {
            if let Some(value) = line.split_once(delim) {
                let (key, path) = value;
                let aligned = tab_align(key, width, delim);
                trace!("{}", &aligned);
                let output = format!("{}{}\n", aligned, path);
                trace!("{}", &output);
                return output
            }
            format!("{}\n", line)
        }).collect();
        let conts = fileconts.join("").strip_suffix('\n').unwrap().to_owned();

        let result = write_config(&self.cfg, CONFIG_FILE_PATH, &conts);
        if let Err(e) = result {
            eprintln!("Failed to write shortpaths config to disk: {}", e);
        }
        println!("Wrote shortpaths config to {}", self.cfg.files.get(CONFIG_FILE_PATH).unwrap().display());
    }
}

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
