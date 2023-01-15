use shortpaths::config::AppConfig;
use shortpaths::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
};

//use std::{
    //path::Path,
//};

//use shortpaths::config::{
    //CONFIG_FILE_PATH,
    //ShortpathsConfig,
    //get_config_path, make_config_dir};

use clap::{arg, ArgAction, Command};

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
            .arg(arg!(-n --name "Remove shortpath by name"))
            .arg(arg!(-p --path "Remove shortpath by path")),
            )
        .subcommand(
            Command::new("check")
            .about("Checks all shortpaths")
            )
        .subcommand(
            Command::new("update")
            .about("Update a shortpath")
            .arg(arg!([CURRENT_NAME]).required(true))
            .arg(arg!(-n --name "New shortpath name"))
            .arg(arg!(-p --path "New shortpath path")),
        );
    cli
}

fn main() {
    pretty_env_logger::init();
    let matches = build_cli().get_matches();

    // Setup initial configs
    let app_cfg = AppConfig::default();
    //let cfg_fp = get_config_path(CONFIG_FILE_PATH);
    //let cfg_path = Path::new(&cfg_fp);
    //if !cfg_path.exists() {
        //make_config_dir();
        //make_default_config();
        //// TODO Create the initial toml config file
    //}

    // TODO:
    // lib.rs: config
    // 1. Create basic toml config
    // 2. Read the basic toml config
    // 3. Write to basic toml config

    // lib.rs: shortpaths
    // 1. Make add, remove, check, update functions

    // lib.rs: export
    // 1. make_shell_completions function to generate bash shell completions

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let (alias_name, alias_path) = (
                sub_matches.get_one::<String>("ALIAS_NAME").unwrap().to_owned(),
                sub_matches.get_one::<String>("ALIAS_PATH").unwrap().to_owned());
            
            
            //let sp = Shortpath::new(alias_name, PathBuf::from(alias_path));
            

            // If there is an existing shortpath file, append the things to it/load it and overwrite
            // Else make new directory and file
        }
        Some(("remove", sub_matches)) => {
        }
        Some(("check", sub_matches)) => {
            // For all shortpaths
            // Check that the path name exist, if it does not, then attempt to find
            // Go through every path in order even if they fail
        }
        Some(("update", sub_matches)) => {
            // Change existing 
        }
        _ => {}
    }

}
