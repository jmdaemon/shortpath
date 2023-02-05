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
use shortpaths::app::{build_cli, toggle_logging, Shortpaths};

use std::{
    path::PathBuf,
    process::exit,
};

use log::info;



fn main() {
    let matches = build_cli().get_matches();
    toggle_logging(&matches);

    // Setup initial configs
    let shortpaths_config = Shortpaths::new();
    //let mut config = Config::new();
    //let cfg_name = CONFIG_FILE_PATH.to_string();
    //let cfg_path = config.format_config_path(&cfg_name);
    //config.add_config(cfg_name, cfg_path.to_str().unwrap());
    
    // Toml String
    //let toml_conts = read_config(&config);
    info!("Current App Shortpaths:\n{}", toml::to_string_pretty(&shortpaths_config).expect("Could not serialize."));

    // TODO: Serde this instead, but for now, pretend its the config
    //let sp_paths = vec![
        //Shortpath::new(SPT::new_path("d", PathBuf::from("$a/dddd")), None, None),
        //Shortpath::new(SPT::new_path("c", PathBuf::from("$b/cccc")), None, None),
        //Shortpath::new(SPT::new_path("b", PathBuf::from("$a/bbbb")), None, None),
        //Shortpath::new(SPT::new_path("a", PathBuf::from("aaaa")), None, None),
    //];
    //let mut sp_builder = ShortpathsBuilder::new(sp_paths);

    //let mut shortpaths = sp_builder.build().unwrap();
    //let mut shortpaths = shortpaths_config.paths;
    let mut shortpaths = shortpaths_config.shortpaths;

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
