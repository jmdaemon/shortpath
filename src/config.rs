//use crate::shortpaths::{ShortpathsConfig, Shortpaths};
use crate::shortpaths::{ShortpathsConfig};
use crate::consts::{
    QUALIFIER,
    ORGANIZATION,
    APPLICATION,
    CONFIG_FILE_PATH,
};

use std::{
    fs,
    collections::HashMap,
    path::PathBuf,
};

use directories::ProjectDirs;

pub struct App {
    pub path: PathBuf,
    pub sp_cfg: ShortpathsConfig,
}

impl App {

    pub fn new(path: PathBuf, sp_cfg: ShortpathsConfig) -> App {
        App { path, sp_cfg }
    }

    pub fn default() -> App {
        let path = get_config_path();
        let sp_cfg = ShortpathsConfig { shortpaths: HashMap::new() };
        //let sp_cfg = ShortpathsConfig { shortpaths: Shortpaths { shortpaths: HashMap::new() } };
        //let sp = Shortpaths { aliases: HashMap::new() };
        //let sp_cfg = ShortpathsConfig { shortpaths: sp };
        let mut app = App::new(path, sp_cfg);
        app.init();
        app
        //if !path.exists() {
            //make_config_dir();
            //make_default_cfg();
        //}
        //App { path }
    }

    /// Loads the app config if it exists, and uses the default config otherwise
    pub fn init(&mut self) {
        //let app_cfg = App::default();
        //let path = get_config_path();
        if self.path.exists() {
            //load_config();

            // Load config
            let toml_conts = fs::read_to_string(&self.path).expect(&format!("Could not read file: {}", self.path.display()));
            let sp_cfg: ShortpathsConfig = toml::from_str(&toml_conts).expect("Could not deserialize shortpaths");
            self.sp_cfg = sp_cfg;

        } else {
            // Initialize config
            make_config_dir();
            let toml_conts = toml::to_string_pretty(&self.sp_cfg).expect("Could not serialize shortpaths");
            fs::write(&self.path, toml_conts).expect("Unable to write file");

            //make_default_cfg(self);
        }
    }
}

pub fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap()
}

pub fn make_config_dir() {
    let proj_dirs = get_project_dirs();
    let cfg_dir = proj_dirs.config_dir();
    fs::create_dir_all(cfg_dir).expect("Could not create config directory");
}

// Create a shortpath and serialize the shortpath
//pub fn make_default_cfg(app: &App) {
    ////todo!("Implement toml::to_string_pretty(&shortpaths_cfg)");
    //let toml_conts = toml::to_string_pretty(&app.sp_cfg).expect("Could not serialize shortpaths");
    //fs::write(&app.path, toml_conts).expect("Unable to write file");
//}

//pub fn load_config() -> ShortpathsConfig {
    //let path = get_config_path();
    //let toml_conts = fs::read_to_string(&path).expect(&format!("Could not read file: {}", path.display()));
    //let sp_cfg: ShortpathsConfig = toml::from_str(&toml_conts).expect("Could not deserialize shortpaths");
    //sp_cfg
//}

pub fn fmt_config_path(config: &str) -> PathBuf {
    let proj_dirs = get_project_dirs();
    proj_dirs.config_dir().to_path_buf().join(config)
}

pub fn get_config_path() -> PathBuf {
    fmt_config_path(CONFIG_FILE_PATH)
}

//pub fn get_config_path(cfg: &str) -> String {
    //let proj_dirs = get_project_dirs();
    //format!("{}/{}", proj_dirs.config_dir().to_str().unwrap(), cfg)
//}
