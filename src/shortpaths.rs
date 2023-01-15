use std::{
    path::{Path, PathBuf},
    collections::HashMap,
};

use log::{debug, trace};
use serde::{Serialize, Deserialize};
use walkdir::{DirEntry, WalkDir};
use directories::UserDirs;

// Data Structures
// Loading & Saving Shortpaths

#[derive(Serialize, Deserialize, Debug)]
pub struct ShortpathsConfig {
    //shortpaths: Vec<Shortpath>,
    pub shortpaths: HashMap<String, PathBuf>,
    //pub shortpaths: Shortpaths,
}

//#[derive(Serialize, Deserialize, Debug)]
//pub struct Shortpaths {
    //#[serde(serialize_with = "toml::ser::tables_last")]
    //pub aliases: HashMap<String, PathBuf>,
//}

//struct Shortpath {
    //name: String,
    //path: PathBuf,
//}

//pub fn read_shortpath_cfg(p: &Path) -> String {
    //fs::read_to_string(p).expect(&format!("Could not read file: {}", p))
//}

//pub fn write_shortpath_cfg(p: &Path, conts: String) -> String {
    //fs::write(p, conts).expect("Unable to write file");
//}

//pub fn load_shortpaths_cfg(p: &Path) -> ShortpathsConfig {
    //let conts = read_shortpath_cfg(p);
    ////let toml_conts = toml::from_str(&conts).expect(&format!("Could not parse toml config: {}", p));
    //let sp: ShortpathsConfig = toml::from_str(&conts).expect(&format!("Could not parse toml config: {}", p));
    //sp
//}

//pub fn add_shortpath(sp_cfg: ShortpathsConfig, sp: Shortpath) {
    //let (name, path) = sp;
    //sp_cfg.shortpaths.insert(name, path); // Updates old paths
//}

//pub fn remove_shortpath_by_name(sp_cfg: ShortpathsConfig, name: String) {
    //sp_cfg.shortpaths.remove(&name);
//}


//fn find_keys_for_value(map: &HashMap<String, PathBuf>, value: PathBuf) -> Vec<String, PathBuf> {
    //map.iter()
        //.filter_map(|(key, &val)| if val == value { Some(key) } else { None })
        //.collect()
//}

//pub fn remove_shortpath_by_path(sp_cfg: ShortpathsConfig, path: &Path) {
    //let names = find_keys_for_value(&sp_cfg.shortpaths, path);
    //for name in names {
        //remove_shortpath_by_name(sp_cfg, name);
    //}
    //// Return config
//}

/// Finds the matching path
pub fn is_equals(e: &DirEntry, p: &Path) -> bool {
    //if e.path() == p { true } else { false }

    let actual_fname = e.path().file_name().unwrap();
    let sp_fname = p.file_name().unwrap();

    trace!("actual_fname: {}", actual_fname.clone().to_str().unwrap());
    trace!("sp_fname: {}", sp_fname.clone().to_str().unwrap());

    //if e.file_name() == p.file_name().unwrap() { true } else { false }
    if actual_fname == sp_fname { true } else { false }
}

////pub fn find_matching_path(shortpath: &Path) -> String {
// TODO: Ensure the searching is done by using the file stem of the original path
pub fn find_matching_path(shortpath: &Path) -> PathBuf {
    let user_dirs = UserDirs::new().expect("No valid home directory found.");
    let home = user_dirs.home_dir();

    let mut next = shortpath;

    // If we match the current user's home, then we exit early
    // TODO: This could lead to bugs in the future if the directory isn't a subdir of $HOME

    //if next == home {
        //eprintln!("Could not find directory: {}", shortpath.to_str().unwrap());
        ////return shortpath.to_str().unwrap().to_string();
        //return shortpath.to_path_buf();
    //}
    
    //let mut new_path = String::new();
    let mut new_path = PathBuf::new();
    //while next.parent().unwrap() != shortpath {
    //while next.parent().unwrap() != home {
    while next.parent().unwrap() != shortpath {
        // Check if parent directory contains any files that match the shortpath
        //let mut files = WalkDir::new(next);

        // Get all files within the next directory
        debug!("Getting list of files of directory {}", next.display());
        let mut parent_files = WalkDir::new(next).max_depth(1);
        //let mut parent_files = WalkDir::new(next);
        
        // Check if any of these files match our given file name
        debug!("Searching for matching file names");
        let mut files: Vec<DirEntry> = vec![];
        for file in parent_files {
            if let Ok(e) = file {
                if is_equals(&e, shortpath) {
                    files.push(e);
                }
            }
            //let e = file.unwrap();
            //if is_equals(&e, shortpath) {
                //files.push(e);
            //}
        }
        //let mut files: Vec<DirEntry> = parent_files.into_iter()
            //.filter(|e| {
                //let e = e.unwrap();
                //is_equals(&e, next.parent().unwrap())
        //}).collect();

        //let mut files = WalkDir::new(next)
            //.into_iter()
            ////.filter_entry(|e| is_equals(e, next.parent().unwrap()))
            //.filter_entry(|e| is_equals(e, shortpath))
            //.filter_map(|v| v.ok());

        // Get first matching result
        //let first = files.next();
        let first = files.first();
        
        // Return the shortpath if it exists
        match first {
            Some(path) => {
                //new_path = path.path().to_str().unwrap().to_owned();
                new_path = path.path().to_path_buf();
                debug!("Match Found: {}", new_path.display());

                // TODO: Set alias's path to new path

                //return path.path().to_str().unwrap().to_owned()
                break;
            }
            None => {
                // Continue searching
                next = next.parent().unwrap();
            }
        }
    }
    //eprintln!("Could not find directory: {}", shortpath.to_str().unwrap());

    if let None = new_path.to_str() {
        eprintln!("Could not find directory: {}", shortpath.to_str().unwrap());
        eprintln!("Unsetting shortpath");
    }
    new_path
}



//impl ShortpathsConfig {
    // TODO: Make new function that initializes the file

    //pub fn read(&self, p: &Path) -> String {
        //fs::read_to_string(p).expect(&format!("Could not read file: {}", p))
    //}

    //// Appends shortpath to current shortpath configs
    //pub fn add(cfg: String, sp: Shortpath) {
        //// TODO: Read config, index into cfg, change line, serialize
    //}
//}

// TODO: Think about whether to store the path as an option
// TODO: Think about adding no_modify, no_remove Option<bool> vars



//impl Shortpath {
    //pub fn new(name: String, path: PathBuf) -> Shortpath {
        //Shortpath { name, path }
    //}


//}

