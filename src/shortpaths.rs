use crate::consts::CONFIG_FILE_PATH;
use crate::config::Config;
use crate::export::get_exporter;

use std::{
    fs,
    path::{Path, PathBuf, Component},
    collections::HashMap,
};

use bimap::{BiHashMap, Overwritten};
use serde::{Serialize, Deserialize};
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
    let path = &config.files;
    let shortpaths_toml = path.get(CONFIG_FILE_PATH).expect("Unable to retrieve path from files");

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
        let shortpaths_toml = config.files.get(CONFIG_FILE_PATH).unwrap();
        let shortpaths = match shortpaths_toml.exists() {
            true => read_shortpaths_from_disk(&config).unwrap(),
            false => BiHashMap::new()
        };

        App { config, shortpaths }
    }

    /// Writes shortpaths to disk
    pub fn save_to_disk(&self) {
        let config = &self.config.files;
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
    pub fn autoindex(&self, on_update: Option<fn(&String, &Path, &Path) -> ()>) -> Shortpaths {
        info!("Finding unreachable shortpaths");
        let on_none = |result: &PathBuf, shortpath: &Path| {
            if let None = result.to_str() {
                eprintln!("Could not find directory: {}", shortpath.to_str().unwrap());
            }
        };
        let mut shortpaths: Shortpaths = BiHashMap::new();
        for (alias_name, alias_path) in &self.shortpaths {
            let shortpath = match alias_path.exists() {
                true => fold_shortpath(alias_name, alias_path, &self.shortpaths),
                false => {
                    // FIXME Too many layers of indentation
                    let expanded_maybe = expand_shortpath(alias_path, &self.shortpaths);
                    if let Some(expanded) = expanded_maybe {
                        let matching = find_matching_path(&expanded);
                        let return_path = match matching {
                            Some(path) => fold_shortpath(alias_name, &path, &self.shortpaths),
                            None => {
                                on_none(&alias_path, &alias_path);
                                alias_path.to_path_buf() // Don't change it
                            }
                        };
                        return_path
                    } else {
                        alias_path.to_path_buf() // Don't change it
                    }
                }
            };
            if let Some(on_update) = on_update {
                on_update(alias_name, alias_path, shortpath.as_path());
            }
            shortpaths.insert(alias_name.clone(), shortpath);
        }
        shortpaths
    }

    /** Serialize shortpaths to other formats for use in other applications */
    pub fn export(&self, export_type: &str, output_file: Option<&String>) -> String {
        let mut exp = get_exporter(export_type.into());
        exp.set_shortpaths(&self.shortpaths);
        
        let dest = match output_file {
            Some(path)  => Path::new(path).to_path_buf(),
            None        => PathBuf::from(exp.get_completions_path())
        };

        fs::create_dir_all(dest.parent().expect("Could not get parent directory"))
            .expect("Could not create shell completions directory");

        // Serialize
        let output = exp.gen_completions();
        fs::write(&dest, &output).expect("Unable to write to disk");
        dest.to_str().unwrap().to_owned()
    }

    /** Update a single shortpath's alias name or path
      * Changes the name or path if given and are unique */
    pub fn update(&mut self, current_name: &str, alias_name: Option<&String>, alias_path: Option<&String>) {
        if let Some(new_path) = alias_path {
            let path = PathBuf::from(new_path);
            if &path != self.shortpaths.get_by_left(current_name).unwrap() {
                self.shortpaths.insert(current_name.to_owned(), PathBuf::from(new_path));
            }
        } else if let Some(new_name) = alias_name {
            if current_name != new_name {
                let path = self.shortpaths.remove_by_left(current_name).unwrap().1;
                self.shortpaths.insert(new_name.to_owned(), path);
            }
        } 
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
pub fn expand_shortpath(path: &Path, spaths: &Shortpaths) -> Option<PathBuf> {
    trace!("Attempting to expand path: {}", path.to_str().unwrap());
    for component in path.components() {
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
                let output = path.to_str().unwrap().to_string().replace(&this, with);
                trace!("Expanded shortpath to: {}", output);
                return Some(PathBuf::from(output))
            }
        }
    }
    None
}

/** Folds nested shortpaths */
pub fn fold_shortpath(alias: &String, shortpath: &Path, spaths: &Shortpaths) -> PathBuf {
    let mut output = shortpath.to_str().unwrap().to_string();
    trace!("Attempting to fold path: {}", output);

    for (alias_name, alias_path) in spaths {
        trace!("Alias Name: {}", alias_name);
        if alias_name == alias { break; } // Don't fold the shortpath with itself
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
pub fn find_matching_path(shortpath: &Path) -> Option<PathBuf> {
    let search_term = shortpath.file_name().unwrap();
    let mut next = shortpath.parent();

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
        if let Some(path) = files.first() {
            let new_path = path.path().to_path_buf();
            debug!("Match Found: {}", new_path.display());
            return Some(new_path);
        }
        next = dir.parent(); // Continue searching
    }
    None
}
