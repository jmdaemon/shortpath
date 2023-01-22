use crate::consts::{
    QUALIFIER,
    ORGANIZATION,
    APPLICATION,
    CONFIG_FILE_PATH,
};

//use core::slice::SlicePattern;
use std::{
    fs,
    path::{Path, PathBuf, Component},
    collections::HashMap,
    ffi::OsStr, fmt::format,
};

use bimap::BiHashMap;
use serde::{Serialize, Deserialize};
use derivative::Derivative;
use directories::ProjectDirs;

use log::{debug, trace};
use walkdir::{DirEntry, WalkDir};

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
// End of config

/// Determines if two paths share the same name
pub fn has_equal_fname(e: &DirEntry, p: &Path) -> bool {
    let entry_fname = e.path().file_name().unwrap();
    let search_fname = p.file_name().unwrap();
    trace!("Entry File Name : {}", entry_fname.clone().to_str().unwrap());
    if entry_fname == search_fname { true } else { false }
}

/** Destructures and returns an alias name if there is one */
pub fn get_alias_name(path: &[char]) -> Option<String> {
    match path {
        ['$', alias_name @ ..] => { Some(alias_name.iter().collect()) }
        _ => { None }
    }
}

/** Expands nested shortpaths definitions
  *
  * If you have the following config:
  *     test = '/home/user/test'
  *     mypath = '$test/mypath'
  * 
  * The shortpaths will be expanded to:
  *     test = '/home/user/test'
  *     mypath = '/home/user/test/mypath'
  *
  * NOTE that this function only expands one alias at a time i.e
  *     test = '/home/user/test'
  *     a = '$test/a'
  *     b = '$a/b'
  *     c = '$a/c'
  * Will expand b to '$test/a' and not '/home/user/test/a/b'
  */
pub fn expand_shortpath(path: &Path, spaths: &Shortpaths) -> PathBuf {
    let mut output = path.to_str().unwrap().to_string();
    trace!("Attempting to expand path: {}", output);

    let iter = path.components().peekable();
    for component in iter {
        trace!("Path Component: {}", component.as_os_str().to_str().unwrap());
        if let Component::Normal(path_component) = component {
            let spath = path_component.to_str().unwrap().to_string();
            let chars: Vec<char> = spath.chars().collect();
            let alias_name = get_alias_name(chars.as_slice());

            trace!("Alias Name: {:?}", alias_name);
            match alias_name {
                Some(alias) => {
                    let nested_path = spaths.get_by_left(&alias).unwrap();

                    // Expands '$aaaa/path' -> '/home/user/aaaa/path'
                    let this = format!("${}", &alias);
                    let with = nested_path.to_str().to_owned().unwrap();
                    output = output.replace(&this, with);
                    trace!("Expanded shortpath to: {}", output);
                }
                None => {}
            }
        }
    }
    PathBuf::from(output)
}

/// Folds nested shortpaths, environment variables
pub fn fold_shortpath(path: &Path, spaths: &Shortpaths) -> PathBuf {
    let mut output = String::from(path.to_str().unwrap());
    let current_key = path.file_name().unwrap().to_str().to_owned().unwrap().to_owned();
    trace!("current_key: {}", current_key);

    // Get the shortpath alias associated with this path
    for (k, v) in spaths.into_iter() {
        let key = spaths.get_by_right(path);
        trace!("Key: {:?}", key);
        if output.contains(&v.to_str().unwrap()) {
            if k != &current_key {
                let nested_alias_name = format!("${}", k);
                trace!("nested_alias_name: {}", nested_alias_name);

                let nested_alias_path = String::from(v.to_str().unwrap());
                trace!("nested_alias_path: {}", nested_alias_path);

                let alias_path = String::from(path.to_str().unwrap());

                let (_, alias_subdir) = alias_path.split_once(&nested_alias_path).unwrap();
                trace!("alias_subdir: {}", alias_subdir);

                let replace_with = format!("{}{}", nested_alias_name, alias_subdir);
                output = output.replace(&output, &replace_with);
                trace!("output: {}", output);
            }
        }
    }
    PathBuf::from(output)
}

/**
  * Searches for matching file paths
  * 
  * Note that the file name of the original shortpath is used to find
  * the file and return its new updated path. If the file/folder is renamed/deleted
  * then it will not be possible to automatically find the 'new' path.
  *
  * Whenever we rename files, we add a hook to call the `shortpaths update` command, and
  * when we delete files, we add a hook to rm  to call `shortpaths remove`
  */
pub fn find_matching_path(shortpath: &Path, spaths: &Shortpaths) -> PathBuf {
    // Expand nested shortpaths
    let expanded = expand_shortpath(shortpath, spaths);
    //trace!("Expanded shortpath: {}", expanded.display());

    let mut next = expanded.as_path();

    let mut new_path = PathBuf::new();
    while next.parent().unwrap() != expanded {
        // Check if the next directory contains any files that match our old path filename
        debug!("Getting list of files of directory {}", next.display());
        let parent_files = WalkDir::new(next).max_depth(1);
        
        // Check if any of these files match our given file name
        debug!("Searching for matching file names");
        let mut files: Vec<DirEntry> = vec![];
        for file in parent_files {
            if let Ok(e) = file {
                if has_equal_fname(&e, expanded.as_path()) {
                    files.push(e);
                }
            }
        }

        // Get first matching result
        let first = files.first();
        
        // Return the shortpath if it exists
        match first {
            Some(path) => {
                new_path = path.path().to_path_buf();
                new_path = fold_shortpath(&new_path, spaths); // Fold shortpaths into aliases
                debug!("Match Found: {}", new_path.display());
                break;
            }
            None => {
                // Continue searching
                next = next.parent().unwrap();
            }
        }
    }

    if let None = new_path.to_str() {
        eprintln!("Could not find directory: {}", shortpath.to_str().unwrap());
        eprintln!("Unsetting shortpath");
    }
    new_path
}
