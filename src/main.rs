use shortpath::app::{create_logger, CLI, Commands};
use shortpath::builder::{ShortpathsBuilder, ShortpathOperationsExt, to_disk};
use shortpath::consts::CONFIG_FILE_PATH;
use shortpath::shortpaths::{
    add_shortpath,
    remove_shortpath,
    check_shortpaths,
    resolve,
    export_shortpaths,
    update_shortpath,
    show_shortpaths,
};

use std::process::exit;

use log::info;
use clap::Parser;

fn main() {
    let cli = CLI::parse();
    if cli.verbose {
        create_logger();
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
        Some(Commands::Remove { names, yes}) => {
            let removed = remove_shortpath(&mut shortpaths, names.as_slice(), yes);
            paths.shortpaths = shortpaths;
            for (name, sp) in names.iter().zip(removed.into_iter()) {
                let sp = sp.unwrap();
                println!("Removed {}: {}", name, sp.path.display());
            }
        }
        Some(Commands::Check {  }) => {
            check_shortpaths(&mut shortpaths);
        }
        Some(Commands::Show { names }) => {
            show_shortpaths(&paths, names);
        }
        Some(Commands::Resolve { resolve_type, mode, dry_run }) => {
            resolve(&mut shortpaths, resolve_type, mode, dry_run);
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
    paths.shortpaths.sort_paths_inplace();
    to_disk(paths);
}
