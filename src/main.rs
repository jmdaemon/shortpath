use shortpaths::app::{build_cli, toggle_logging, Shortpaths};
use shortpaths::shortpaths::{
    add_shortpath,
    remove_shortpath,
    check_shortpaths,
    resolve,
    export_shortpaths,
    update_shortpath,
};

use std::{
    path::PathBuf,
    process::exit,
};

use log::info;

fn main() {
    let matches = build_cli().get_matches();
    toggle_logging(&matches);

    // Setup initial configs
    let mut shortpaths_config = Shortpaths::new();

    info!("Current App Shortpaths:\n{}", toml::to_string_pretty(&shortpaths_config).expect("Could not serialize."));

    let mut shortpaths = shortpaths_config.shortpaths.clone();

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let (alias_name, alias_path) = (
                sub_matches.get_one::<String>("NAME").unwrap().to_owned(),
                sub_matches.get_one::<String>("PATH").unwrap()
            );

            add_shortpath(&mut shortpaths, alias_name.clone(), PathBuf::from(alias_path));
            println!("Saved shortpath {}: {}", alias_name, alias_path);
        }
        Some(("remove", sub_matches)) => {
            let current_name = sub_matches.get_one::<String>("NAME").unwrap();
            let path = remove_shortpath(&mut shortpaths, current_name);
            println!("Removed {}: {}", current_name.to_owned(), path.unwrap().path.display());
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
            println!("Exported shell completions to {}", dest.display());
        }
        Some(("update", sub_matches)) => {
            let (current_name, alias_name, alias_path) = (
                sub_matches.get_one::<String>("CURRENT_NAME").unwrap(),
                sub_matches.get_one::<String>("NAME"),
                sub_matches.get_one::<String>("PATH"),
            );

            if alias_name.is_none() && alias_path.is_none() {
                println!("Shortpath name or path must be provided");
                exit(1);
            }
            update_shortpath(&mut shortpaths, current_name, alias_name, alias_path);
        }
        _ => {}
    }
    shortpaths_config.to_disk();
}
