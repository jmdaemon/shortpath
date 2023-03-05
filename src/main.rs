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
        Some(Commands::Hook { hook }) => {
            // NOTE: These will fail since the find_key_for_value function does
            // not take into account that the full_path doesn't always == path
            match hook {
                Some(Hooks::Remove { names }) => {
                    if names.is_none() {
                        exit(1);
                    }
                    info!("{:?}", names);

                    let names: Vec<String> = names.unwrap().into_iter().filter(|p| {
                        shortpaths.find_key_for_value(p).is_none()
                    })
                    .map(|p| String::from(fold_shortpath(PathBuf::from(p), &shortpaths).to_str().unwrap()))
                    .collect();
                    info!("{:?}", names);

                    let removed = remove_shortpath(&mut shortpaths, names.as_slice(), true);
                    paths.shortpaths = shortpaths;
                    info!("{:?}", removed);
                    for (name, sp) in names.iter().zip(removed.into_iter()) {
                        let sp = sp.unwrap();
                        println!("Removed {}: {}", name, sp.path.display());
                    }
                }
                Some(Hooks::Move { src, dest }) => {
                    println!("Given: {}", src.display());
                    println!("Given: {}", dest.display());
                    // Use the fully qualified paths
                    let src = src.canonicalize().unwrap();
                    println!("{}", src.display());

                    let mut new_dest = src.clone();
                    new_dest.pop();
                    new_dest.push(dest);
                    let dest = new_dest;
                    //let dest = dest.canonicalize().unwrap();
                    println!("{}", dest.display());

                    let spclone = shortpaths.clone(); 
                    // Expand all the shortpaths
                    //let p = fold_shortpath(PathBuf::from(src), &shortpaths);
                    let folded = fold_shortpath(src, &shortpaths);
                    let folded = folded.to_str().unwrap();
                    println!("folded: {}", folded);
                    //let folded = {
                        //let p = fold_shortpath(PathBuf::from(src), &shortpaths);
                        //p.to_str().unwrap().to_owned()
                    //};
                    //let folded = {
                        //p.to_str().unwrap()
                    //};
                    let key = spclone.find_key_for_value(folded);
                    println!("key: {:?}", key);
                    if let Some(key) = key {
                        //update_shortpath(&mut shortpaths, key, Some(key.to_owned()), Some(PathBuf::from(dest)));
                        //update_shortpath(&mut shortpaths, key, Some(key.to_owned()), Some(dest));
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
}
