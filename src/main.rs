use shortpath::app::{create_logger, CLI, Commands, Hooks};
use shortpath::builder::{ShortpathsBuilder, ShortpathOperationsExt, to_disk};
use shortpath::consts::CONFIG_FILE_PATH;
use shortpath::shortpaths::{
    add_shortpath,
    remove_shortpath,
    check_shortpaths,
    resolve,
    export_shortpaths,
    update_shortpath,
    show_shortpaths, FindKeyIndexMapExt, fold_shortpath, update_shortpath_path,
};

use std::path::PathBuf;
use std::process::exit;

use log::{info, debug};
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
        Some(Commands::Hook { hook }) => {
            // NOTE: These will fail since the find_key_for_value function does
            // not take into account that the full_path doesn't always == path
            match hook {
                Some(Hooks::Remove { filepaths }) => {
                    // Exit if nothing was passed
                    if filepaths.is_none() {
                        exit(1);
                    }
                    println!("Given: {:?}", filepaths);

                    // Get the fully qualified file paths for the inputs
                    let filepaths = filepaths.unwrap();
                    let filepaths: Vec<PathBuf> = filepaths.into_iter().map(|path| {
                        path.canonicalize().unwrap()
                    }).collect();
                    println!("Canonicalized Paths: {:?}", filepaths);

                    // Filter only shortpath definitions
                    let filepaths: Vec<PathBuf> = filepaths.into_iter().filter(|path| {
                        let path = path.to_str().unwrap();
                        shortpaths.find_key_for_value(path).is_none()
                    }).collect();
                    println!("Filtered Shortpaths: {:?}", filepaths);

                    // Fold the resulting shortpaths
                    let filepaths: Vec<String> = filepaths.into_iter().map(|p| {
                        let folded = fold_shortpath(p, &shortpaths);
                        let folded = folded.to_str().unwrap();
                        String::from(folded)
                    }).collect();
                    println!("Folded Shortpaths: {:?}", filepaths);

                    // Remove shortpath definitions
                    let names: Vec<String> = filepaths.into_iter().filter_map(|path| {
                        let key = shortpaths.find_key_for_value(path);
                        println!("{:?}", key);
                        if let Some(key) = key {
                            return Some(key.to_owned())
                        }
                        None
                    }).collect();
                    println!("Key Names of Shortpaths: {:?}", names);
                    let removed = remove_shortpath(&mut shortpaths, names.as_slice(), true);
                    paths.shortpaths = shortpaths;
                    println!("{:?}", removed);

                    // Display results to user
                    for (name, sp) in names.iter().zip(removed.into_iter()) {
                        let sp = sp.unwrap();
                        println!("Removed {}: {}", name, sp.path.display());
                    }
                }
                Some(Hooks::Move { src, dest }) => {
                    debug!("Given: {}", src.display());
                    debug!("Given: {}", dest.display());
                    // Get the fully qualified file paths
                    let src = src.canonicalize().unwrap();
                    debug!("{}", src.display());

                    let mut new_dest = src.clone();
                    new_dest.pop();
                    new_dest.push(dest);
                    let dest = new_dest;
                    debug!("{}", dest.display());

                    // Check if the path exists in the shortpaths config
                    // Requires the folded variant of the full_path
                    let spclone = shortpaths.clone(); 
                    let folded = fold_shortpath(src, &shortpaths);
                    let folded = folded.to_str().unwrap();
                    debug!("folded: {}", folded);

                    // Update the shortpath definition
                    let key = spclone.find_key_for_value(folded);
                    debug!("key: {:?}", key);
                    if let Some(key) = key {
                        let folded = fold_shortpath(dest.clone(), &shortpaths);
                        update_shortpath_path(key, folded, Some(dest), &mut shortpaths);
                        paths.shortpaths = shortpaths;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    paths.shortpaths.sort_paths_inplace();
    to_disk(paths);
    exit(0);
}
