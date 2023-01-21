use crate::consts::{
    QUALIFIER,
    ORGANIZATION,
    APPLICATION,
    CONFIG_FILE_PATH,
};

use std::{
    fs,
    path::PathBuf,
    collections::HashMap,
};

use bimap::BiHashMap;
use serde::{Serialize, Deserialize};
use derivative::Derivative;
use directories::ProjectDirs;

pub type Shortpaths = BiHashMap<String, PathBuf>;

#[derive(Serialize, Deserialize, Debug)]
pub struct App {
    #[serde(skip)]
    pub config: Config,
    pub shortpaths: Shortpaths,
}

/** Reads the shortpaths.toml configuration from disk if it exists */
pub fn read_shortpaths_from_disk(config: &Config) -> Option<Shortpaths> {
    let path = &config.config_files;
    let shortpaths_toml = path.get(CONFIG_FILE_PATH).expect("Unable to retrieve path from config_files");

    if shortpaths_toml.exists() {
        let toml_conts = fs::read_to_string(shortpaths_toml)
            .expect(&format!("Could not read file: {}", shortpaths_toml.display()));
        let app: App = toml::from_str(&toml_conts).expect("Could not deserialize shortpaths");
        Some(app.shortpaths)
    } else {
        None
    }
}

impl App {
    pub fn new() -> App {
        let mut config = Config::new();
        // Insert our needed config files
        config.add_config(CONFIG_FILE_PATH.to_owned(), CONFIG_FILE_PATH);

        // Read shortpaths from disk if it exists 
        let shortpaths_toml = config.config_files.get(CONFIG_FILE_PATH).unwrap();
        let shortpaths = match shortpaths_toml.exists() {
            true => read_shortpaths_from_disk(&config).unwrap(),
            false => BiHashMap::new()
        };

        App { config, shortpaths }
    }

    /// Writes shortpaths to disk
    pub fn save_to_disk(&self) {
        let config = &self.config.config_files;
        let path = config.get(CONFIG_FILE_PATH).unwrap();
        let toml_conts = toml::to_string_pretty(&self).expect("Could not serialize app shortpaths");
        match fs::write(path, toml_conts) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Could not save shortpaths to disk.\n{}", e);
            }
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct Config {
    #[derivative(Default(value = "ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).expect(\"Could not initialize config\")"))]
    proj_dirs: ProjectDirs,
    pub config_files: HashMap<String, PathBuf>
}

impl Config {
    pub fn new() -> Config {
        let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).expect("Could not initialize config");
        let config_files = HashMap::new();

        // Initialize config
        let config = Config { proj_dirs, config_files };
        config.init();
        config
    }

    fn init(&self) {
        let proj_cfg_dir = self.proj_dirs.config_dir();
        fs::create_dir_all(proj_cfg_dir).expect("Could not create config directory")
    }

    pub fn format_config_path(&self, config: &str) -> PathBuf {
        self.proj_dirs.config_dir().to_path_buf().join(config)
    }

    pub fn add_config(&mut self, key: String, file: &str) {
        self.config_files.insert(key, self.format_config_path(file));
    }
}
