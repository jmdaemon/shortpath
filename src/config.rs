use crate::consts::{
    QUALIFIER,
    ORGANIZATION,
    APPLICATION,
    CONFIG_FILE_PATH,
};

use std::{
    fs,
    collections::HashMap,
    convert::TryFrom,
    path::PathBuf,
};

//use serde_with::{serde_as, DisplayFromStr};
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;

type Shortpaths = HashMap<String, PathBuf>;

// TODO: Fix serde serialization of hash map

//#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct App {
    #[serde(skip)]
    pub path: PathBuf,
    //#[serde(serialize_with = "toml::ser::tables_last", deserialize_with = "toml::de::from_str")]
    //#[serde_as(as = "HashMap<String, PathBuf>")]
    //#[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub shortpaths: Shortpaths,
}

impl App {
    pub fn new(path: PathBuf, shortpaths: Shortpaths) -> App {
        App { path, shortpaths }
    }


    pub fn default() -> App {
        let path = get_config_path();
        let mut app = App::new(path, HashMap::new());
        app.init();
        app
    }

    /// Loads the app config if it exists, and uses the default config otherwise
    pub fn init(&mut self) {
        if self.path.exists() {
            // Load config from disk
            //println!("{}", self.path.display());
            let toml_conts = fs::read_to_string(&self.path).expect(&format!("Could not read file: {}", self.path.display()));
            //let toml: toml::Value = toml::from_str(&toml_conts).unwrap();
            //println!("{}", toml);
            let app: App = toml::from_str(&toml_conts).expect("Could not deserialize shortpaths");
            self.shortpaths = app.shortpaths;

        } else {
            // Make new config from disk
            make_config_dir();
            let toml_conts = toml::to_string_pretty(&self).expect("Could not serialize shortpaths");
            fs::write(&self.path, toml_conts).expect("Unable to write file");
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

pub fn fmt_config_path(config: &str) -> PathBuf {
    let proj_dirs = get_project_dirs();
    proj_dirs.config_dir().to_path_buf().join(config)
}

pub fn get_config_path() -> PathBuf {
    fmt_config_path(CONFIG_FILE_PATH)
}
