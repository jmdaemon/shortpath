use crate::consts::{
    QUALIFIER,
    ORGANIZATION,
    APPLICATION,
    CONFIG_FILE_PATH,
};

use std::{
    fs,
    path::PathBuf,
};

use directories::ProjectDirs;

pub struct AppConfig {
    pub path: PathBuf,
}

impl AppConfig {

    pub fn new(path: PathBuf) -> AppConfig {
        AppConfig { path }
    }

    pub fn default() -> AppConfig {
        let path = get_config_path();
        if !path.exists() {
            make_config_dir();
            // TODO Make default file too
        }
        AppConfig { path }
    }

    /// Loads the app config if it exists, and uses the default config otherwise
    pub fn init(&self) {
        let app_cfg = AppConfig::default();
        let path = get_config_path();
        if path.exists() {
            // Load config from disk
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
pub fn make_default_cfg() {
    todo!("Implement toml::to_string_pretty(&shortpaths_cfg)");
}

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
