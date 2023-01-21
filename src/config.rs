use crate::consts::{
    QUALIFIER,
    ORGANIZATION,
    APPLICATION,
    CONFIG_FILE_PATH,
};

use std::{
    fs,
    path::{Path, PathBuf},
    collections::HashMap,
};

use bimap::BiHashMap;
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;

use derivative::Derivative;

pub type Shortpaths = BiHashMap<String, PathBuf>;

#[derive(Serialize, Deserialize, Debug)]
pub struct App {
    #[serde(skip)]
    pub config: Config,
    pub shortpaths: Shortpaths,
}

pub fn read_shortpaths_from_disk(config: &Config) -> Option<Shortpaths> {
    //let path = self.config.config_files;
    let path = &config.config_files;
    let shortpaths_toml = path.get(CONFIG_FILE_PATH).expect("Unable to retrieve path from config_files");

    /* If the file exists
     * 1. Load the file into our shortpaths
     * Else if the file does not exist
     * 1. Immediately serialize to disk
     * - We dont actually need to do this though as long as we have it in memory
     *      It will just be redundant
     */
    if shortpaths_toml.exists() {
        let toml_conts = fs::read_to_string(shortpaths_toml)
            .expect(&format!("Could not read file: {}", shortpaths_toml.display()));
        let app: App = toml::from_str(&toml_conts).expect("Could not deserialize shortpaths");
        Some(app.shortpaths)
        //self.shortpaths = app.shortpaths;
    } else {
        None
    }

    //if shortpaths_toml.exists() {
    //} else {
        //let toml_conts = fs::read_to_string(&self.path).expect(&format!("Could not read file: {}", self.path.display()));
        //let app: App = toml::from_str(&toml_conts).expect("Could not deserialize shortpaths");
        //self.shortpaths = app.shortpaths;
    //}
    
    //for (cfg_fname, cfg_fpath) in self.shortpaths.iter() {
        //match cfg_fpath.exists() {
            //true => {
                //// Load the paths

            //}
            //false => {
                //// Initialize the 
            //}
        //}
    //}

    //if self.path.exists() {
        //// Load config from disk
        //let toml_conts = fs::read_to_string(&self.path).expect(&format!("Could not read file: {}", self.path.display()));
        //let app: App = toml::from_str(&toml_conts).expect("Could not deserialize shortpaths");
        //self.shortpaths = app.shortpaths;
    //} else {
        //// Make new config from disk
        //make_config_dir();
        //let toml_conts = toml::to_string_pretty(&self).expect("Could not serialize shortpaths");
        //fs::write(&self.path, toml_conts).expect("Unable to write file");
    //}
}

impl App {
    //pub fn new(path: PathBuf, shortpaths: Shortpaths) -> App {
        //App { path, shortpaths, config: Config::new() }
    //}

    //pub fn default() -> App {
        //let path = get_config_path();
        //let mut app = App::new(path, BiHashMap::new());
        //app.init();
        //app
    //}

    pub fn new() -> App {
        let mut config = Config::new();
        // Insert our needed config files

        //config.config_files.insert("shortpaths.toml", &Path::new(CONFIG_FILE_PATH));
        config.add_config(CONFIG_FILE_PATH.to_owned(), CONFIG_FILE_PATH);

        // Read shortpaths from disk if it exists 
        let shortpaths_toml = config.config_files.get(CONFIG_FILE_PATH).unwrap();
        
        let shortpaths = match shortpaths_toml.exists() {
            true => read_shortpaths_from_disk(&config).unwrap(),
            false => BiHashMap::new()
        };

        //let shortpaths = BiHashMap::new();
        //let app = App { config, shortpaths };
        //app
        App { config, shortpaths }
    }

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

    //pub fn add_config(file: &str) {
        //config.config_files.insert("shortpaths.toml", &Path::new(CONFIG_FILE_PATH));
    //}

    //fn init(&mut self) {
    //fn read_shortpaths_from_disk(self) -> Option<Shortpaths> {
        //let path = self.config.config_files;


    //// Loads the app config if it exists, and uses the default config otherwise
    //pub fn init(&mut self) {
        //if self.path.exists() {
            //// Load config from disk
            //let toml_conts = fs::read_to_string(&self.path).expect(&format!("Could not read file: {}", self.path.display()));
            //let app: App = toml::from_str(&toml_conts).expect("Could not deserialize shortpaths");
            //self.shortpaths = app.shortpaths;
        //} else {
            //// Make new config from disk
            //make_config_dir();
            //let toml_conts = toml::to_string_pretty(&self).expect("Could not serialize shortpaths");
            //fs::write(&self.path, toml_conts).expect("Unable to write file");
        //}
    //}

    //pub fn save_to_disk(&self) {
        //let toml_conts = toml::to_string_pretty(&self).expect("Could not serialize shortpaths");
        //fs::write(&self.path, toml_conts).expect("Unable to write file");
    //}

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
        //config.config_files.insert("shortpaths.toml", &Path::new(CONFIG_FILE_PATH));
    }
}

//pub fn save_to_disk(path: &Path, conts: &str) {
    //fs::write(&self.path, toml_conts).expect("Unable to write file");
    //let toml_conts = toml::to_string_pretty(&self).expect("Could not serialize shortpaths");
    //fs::write(&self.path, toml_conts).expect("Unable to write file");
//}
