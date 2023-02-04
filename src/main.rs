use shortpaths::config::{Config, read_config};
use shortpaths::sp::{
    add_shortpath,
    ShortpathsBuilder,
    Shortpath,
    SPT,
    remove_shortpath,
    check_shortpaths,
    resolve,
    export_shortpaths,
    update_shortpath,
};
use shortpaths::consts::{
    PROGRAM_NAME,
    VERSION,
    AUTHOR,
    PROGRAM_DESCRIPTION,
    CONFIG_FILE_PATH,
};

use std::{
    path::PathBuf,
    process::exit,
};

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
    let mut config = Config::new();
    let cfg_name = CONFIG_FILE_PATH.to_string();
    let cfg_path = config.format_config_path(&cfg_name);
    config.add_config(cfg_name, cfg_path.to_str().unwrap());
    
    // Toml String
    let toml_conts = read_config(&config);
    info!("Current App Shortpaths:\n{}", toml_conts);

    // TODO: Serde this instead, but for now, pretend its the config
    let sp_paths = vec![
        Shortpath::new(SPT::new_path("d", PathBuf::from("$a/dddd")), None, None),
        Shortpath::new(SPT::new_path("c", PathBuf::from("$b/cccc")), None, None),
        Shortpath::new(SPT::new_path("b", PathBuf::from("$a/bbbb")), None, None),
        Shortpath::new(SPT::new_path("a", PathBuf::from("aaaa")), None, None),
    ];
    let mut sp_builder = ShortpathsBuilder::new(sp_paths);

    let mut shortpaths = sp_builder.build().unwrap();

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let (alias_name, alias_path) = (
                sub_matches.get_one::<String>("ALIAS_NAME").unwrap().to_owned(),
                sub_matches.get_one::<String>("ALIAS_PATH").unwrap()
            );

            add_shortpath(&mut shortpaths, alias_name.clone(), PathBuf::from(alias_path));
            println!("Saved shortpath {}: {}", alias_name, alias_path);
        }
        Some(("remove", sub_matches)) => {
            let current_name = sub_matches.get_one::<String>("ALIAS_NAME").unwrap();
            let path = remove_shortpath(&mut shortpaths, current_name);
            println!("Removed {}: {}", current_name.to_owned(), path.unwrap().path().display());
        }
        Some(("check", _)) => {
            check_shortpaths(&mut shortpaths);
        }
        Some(("resolve", _)) => {
            println!("Resolving any unreachable shortpaths");
            let resolve_type = "matching";
            let automode = true;
            resolve(&mut shortpaths, resolve_type, automode);
        }
        Some(("export", sub_matches)) => {
            let (export_type, output_file) = (
                sub_matches.get_one::<String>("EXPORT_TYPE").unwrap(),
                sub_matches.get_one::<String>("OUTPUT_FILE"),
            );
            let dest = export_shortpaths(&shortpaths, export_type, output_file);
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
            update_shortpath(&mut shortpaths, current_name, alias_name, alias_path);
        }
        _ => {}
    }
    //app.save_to_disk();
    // TODO: Write the app state to disk
}
