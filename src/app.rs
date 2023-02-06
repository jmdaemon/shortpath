use crate::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
    CONFIG_FILE_PATH,
};
use crate::config::{Config, read_config, write_config};
use crate::shortpaths::{SP, Shortpath, populate_shortpaths, sort_shortpaths};
use crate::helpers::{expand_tilde, tab_align, find_longest_keyname};

use std::path::PathBuf;

use indexmap::IndexMap;
use serde::{Serialize, Deserialize};
use log::{LevelFilter, trace};
use pretty_env_logger::formatted_timed_builder;
use clap::{arg, ArgAction, Command, ArgMatches};
 
#[derive(Serialize, Deserialize, Debug)]
pub struct Shortpaths {
    #[serde(rename(serialize = "shortpaths", deserialize = "shortpaths"))]
    pub paths: IndexMap<String, PathBuf>,
    #[serde(skip)]
    pub cfg: Config,
    #[serde(skip)]
    pub shortpaths: SP,
}

pub fn setup_config(file: &str) -> Config {
    let mut config = Config::new();
    let cfg_name = file.to_string();
    config.add_config(cfg_name, CONFIG_FILE_PATH);
    config
}

impl Default for Shortpaths {
    fn default() -> Self {
        let cfg = setup_config(CONFIG_FILE_PATH);
        let toml_conts = read_config(&cfg, CONFIG_FILE_PATH);
        let sp: Shortpaths = toml::from_str(&toml_conts).unwrap();

        let paths = sp.paths;
        let mut shortpaths: SP = paths.iter().map(|(name, path)| {
            let sp = Shortpath::new(name.to_owned(), path.to_owned(), None, None);
            (name.to_owned(), sp)
        }).collect();

        let shortpaths = populate_shortpaths(&mut shortpaths);
        Shortpaths { cfg, paths, shortpaths }
    }
}

impl Shortpaths {
    pub fn new() -> Shortpaths {
        Default::default()
    }

    pub fn to_disk(&mut self) {
        let shortpaths = sort_shortpaths(self.shortpaths.to_owned());
        
        let shortpaths: SP = shortpaths.into_iter().map(|(name, mut sp)| {
            let path = expand_tilde(&sp.path).unwrap();
            sp.full_path = Some(path);
            (name, sp)
        }).collect();

        let length = find_longest_keyname(shortpaths.clone()).len();

        let paths: IndexMap<String, PathBuf> = shortpaths.into_iter().map(|(k, sp)| {
            (k, sp.path)
        }).collect();
        self.paths = paths;

        let fileconts = toml::to_string_pretty(&self).expect("Could not serialize shortpaths");
        let fileconts = fileconts.split('\n');
        
        let t_align = |s: &str, delim: &str| {
            let aligned = format!("{: <length$}{}", s, delim);
            aligned
        };
        let fileconts: Vec<String> = fileconts.into_iter().map(|line| {
            if let Some(value) = line.split_once(" = ") {
                let (key, path) = value;
                let delim = " = ";
                let aligned = t_align(key, delim);
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
