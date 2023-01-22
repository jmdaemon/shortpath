use crate::consts::{
    QUALIFIER,
    ORGANIZATION,
    APPLICATION,
    CONFIG_FILE_PATH,
};

use std::{
    fs,
    path::{Path, PathBuf, Component},
    collections::HashMap,
    ffi::OsStr,
};

use bimap::{BiHashMap, Overwritten};
use serde::{Serialize, Deserialize};
use derivative::Derivative;
use directories::ProjectDirs;

use log::{debug, trace, info};
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

    /// Add a new shortpath
    pub fn add(&mut self,  alias_name: &str, alias_path: &Path) -> Overwritten<String, PathBuf> {
        self.shortpaths.insert(alias_name.into(), alias_path.into())
    }

    /// Remove a shortpath
    pub fn remove(&mut self, current_name: &str) -> PathBuf {
        self.shortpaths.remove_by_left(current_name).unwrap().1
    }

    /// Find unreachable shortpaths
    pub fn check(&self) -> HashMap<&String, &PathBuf> {
       let unreachable: HashMap<&String, &PathBuf> = self.shortpaths
           .iter()
           .filter(|(_, v)| if !v.exists() || v.to_str().is_none() { true } else { false }
               )
           .collect::<HashMap<&String, &PathBuf>>();
        unreachable
    }

    /** Finds unreachable paths and tries to resolve them
      * Expands/folds nested shortpaths */
    pub fn autoindex(&self) -> Shortpaths {
        info!("Finding unreachable shortpaths");
        let mut shortpaths: Shortpaths = BiHashMap::new();
        for (alias_name, alias_path) in &self.shortpaths {
            let alias_path = alias_path.as_path();
            let shortpath = match alias_path.exists() {
                true => fold_shortpath(&alias_path.to_path_buf(), &self.shortpaths),
                false => {
                    // If the path is unreachable
                    let matching = find_matching_path(alias_path, &self.shortpaths);
                    fold_shortpath(&matching, &self.shortpaths)
                }
            };
            shortpaths.insert(alias_name.clone(), shortpath);
        }
        shortpaths
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

/** Destructures and returns an alias name if there is one */
pub fn get_alias_name(path: &[char]) -> Option<String> {
    match path {
        ['$', alias_name @ ..] => { Some(alias_name.iter().collect()) }
        _ => { None }
    }
}

/** Expands nested shortpaths definitions
  *
  * Expands the following shortpath '$test/mypath' -> '/home/user/test/mypath'
  * NOTE that this function only expands one alias at a time i.e
  *     test = '/home/user/test'
  *     a = '$test/a'
  *     b = '$a/b'
  * Will expand b to '$test/a' and not all the way to '/home/user/test/a/b'
  * This could be fixed by generating a proper trie dependency tree upon
  * first reading and parsing the config, and using that as well in expand/fold shortpaths
  * to iterate.
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
            if let Some(alias) = alias_name {
                let nested_path = spaths.get_by_left(&alias).unwrap();

                // Expands '$aaaa/path' -> '/home/user/aaaa/path'
                let this = format!("${}", &alias);
                let with = nested_path.to_str().to_owned().unwrap();
                output = output.replace(&this, with);
                trace!("Expanded shortpath to: {}", output);
            }
        }
    }
    PathBuf::from(output)
}

/** Folds nested shortpaths */
pub fn fold_shortpath(shortpath: &Path, spaths: &Shortpaths) -> PathBuf {
    let mut output = shortpath.to_str().unwrap().to_string();
    
    let search_term = shortpath.file_name().unwrap().to_str().unwrap();
    trace!("Attempting to fold path: {}", output);

    for (alias_name, alias_path) in spaths {
        trace!("Alias Name: {}", alias_name);
        if alias_name == search_term {
            break; // Don't fold an already folded path. In the future this could change but right now it just complicates things
        }
        let nested_path = alias_path.to_str().unwrap();
        if output.contains(nested_path) {
            // Shortens '/home/user/aaaa/path' -> '$aaaa/path'
            let this = nested_path;
            let with = format!("${}", &alias_name);
            output = output.replace(&this, &with);
            trace!("Folded shortpath to: {}", output);
        }
    }
    PathBuf::from(output)
}

/** Searches for matching file names
  * 
  * If we have:
  *     test = '/home/user/Workspace/test',
  * and we change the directory on disk but forget to update the shortpath then
  * find_matching_path tries to find it in the parent directory by comparing the file name (e.g 'test')
  * If a match cannot be found, the path is unset.
  *
  * TODO Implement searching by nearest neighbours/parents
  *     - First check nearest neighbours up to 2 sub directories
  *     - If not found, then go up one parent directory
  * TODO Implement fast alternative prolog exhaustive search to find matching
  *     directories. Alternatively, use locate-style finding.
  * TODO Add directories to exclude from search space
  * 
  * NOTE This function isn't used often since we have the shell hooks to update/remove shortpaths
  */
pub fn find_matching_path(shortpath: &Path, spaths: &Shortpaths) -> PathBuf {
    let expanded = expand_shortpath(shortpath, spaths);
    let search_term: &OsStr = expanded.file_name().unwrap();

    let mut next = expanded.as_path().parent();
    let mut new_path = PathBuf::new();

    while let Some(dir) = next {
        debug!("In Directory {}", dir.display());
        let parent_files = WalkDir::new(dir).max_depth(1);

        debug!("Looking for matching name");
        let files: Vec<DirEntry> = parent_files.into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<DirEntry>>().into_iter()
            .map(|f| {
                trace!("File: {}", f.file_name().to_str().unwrap());
                f
            })
            .filter(|file| file.file_name() == search_term).collect();

        // Return the matching path if it exists
        if let Some(path) = files.first(){
            new_path = fold_shortpath(&path.path().to_path_buf(), spaths);
            debug!("Match Found: {}", new_path.display());
            break;
        }
        next = dir.parent(); // Continue searching
    }

    if let None = new_path.to_str() {
        eprintln!("Could not find directory: {}", shortpath.to_str().unwrap());
        eprintln!("Unsetting shortpath");
    }
    new_path
}
