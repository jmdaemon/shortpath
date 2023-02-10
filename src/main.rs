use shortpaths::app::{create_logger, CLI, Commands};
use shortpaths::builder::{ShortpathsBuilder, to_disk};
use shortpaths::consts::CONFIG_FILE_PATH;
use shortpaths::shortpaths::{
    add_shortpath,
    remove_shortpath,
    check_shortpaths,
    resolve,
    export_shortpaths,
    update_shortpath,
    list_shortpaths,
};

use std::process::exit;

use log::info;
//use log::{info, LevelFilter};
//use pretty_env_logger::formatted_timed_builder;
use clap::Parser;

fn main() {
    let cli = CLI::parse();
    if cli.verbose {
        create_logger();
         //formatted_timed_builder().filter_level(LevelFilter::Trace).init();
    }

    let mut paths = ShortpathsBuilder::new()
        .with_config(CONFIG_FILE_PATH)
        .read_shortpaths()
        .build()
        .expect("Could not create shortpaths");

    let mut shortpaths = paths.shortpaths.to_owned();
    info!("Current App Shortpaths:\n{}", toml::to_string_pretty(&shortpaths).expect("Could not serialize."));

    match cli.command {
        Some(Commands::Add { name, path} ) => {
            add_shortpath(&mut shortpaths, name.clone(), path.clone());
            paths.shortpaths = shortpaths;
            println!("Saved shortpath {}: {}", name, path.display());
        }
        Some(Commands::Remove { name }) => {
            let path = remove_shortpath(&mut shortpaths, &name);
            paths.shortpaths = shortpaths;
            println!("Removed {}: {}", name, path.unwrap().path.display());
        }
        Some(Commands::Check {  }) => {
            check_shortpaths(&mut shortpaths);
        }
        Some(Commands::List { names }) => {
            list_shortpaths(&paths, names);
        }
        Some(Commands::Resolve { resolve_type, mode, dry_run }) => {
            println!("Resolving any unreachable shortpaths");
            resolve(shortpaths.clone(), resolve_type, mode, dry_run);
            paths.shortpaths = shortpaths;
        }
        Some(Commands::Export { export_type, output_file }) => {
            println!("{:?}", export_type);
            let dest = export_shortpaths(&shortpaths, export_type, output_file);
            println!("Exported shell completions to {}", dest.display());
        }
        Some(Commands::Update { current_name, name, path }) => {
            if name.is_none() && path.is_none() {
                println!("Shortpath name or path must be provided");
                exit(1);
            }
            update_shortpath(&mut shortpaths, &current_name, name, path);
            paths.shortpaths = shortpaths;
        }
        _ => {}
    }
    to_disk(paths);
}
