use crate::config::Shortpaths;

use std::{
    path::{Path, PathBuf, Component},
    collections::HashMap, fmt::format,
};

use log::{debug, trace};
use serde::{Serialize, Deserialize};
use walkdir::{DirEntry, WalkDir};

#[derive(Serialize, Deserialize, Debug)]
pub struct ShortpathsConfig {
    pub shortpaths: HashMap<String, PathBuf>,
}

/// Determines if two paths share the same name
pub fn has_equal_fname(e: &DirEntry, p: &Path) -> bool {
    let entry_fname = e.path().file_name().unwrap();
    let search_fname = p.file_name().unwrap();

    trace!("Entry File Name : {}", entry_fname.clone().to_str().unwrap());
    trace!("Search File Name: {}", search_fname.clone().to_str().unwrap());
    if entry_fname == search_fname { true } else { false }
}

//pub fn parse_alias(s: String) -> String {
    //s.split_at(1).1.to_string()
//}

pub fn parse_alias(s: String) -> String {
    s.split_at(1).1.to_string()
}

/** Removes the shortpath alias formatting
    This removes the '$' prefix from nested shortpaths */
pub fn parse_alias_from_comp(c: &Component) -> Option<String> {
    let comp = c.as_os_str().to_str().unwrap();
    if comp.starts_with("$") {
        let stripped = String::from(comp).split_at(1).1.to_string();
        Some(stripped)
    } else {
        None
    }
}

//pub fn fmt_alias() -> String {
//}

/// Expands nested shortpaths, environment variables
pub fn expand_shortpath(path: &Path, spaths: &Shortpaths) -> PathBuf {
    let mut output: String = String::from(path.to_str().unwrap());
    // Decompose path components
    for comp in path.components() {
        let compstr = comp.as_os_str().to_str().unwrap();
        let shortpath_name = parse_alias_from_comp(&comp);
        trace!("component: {}", compstr);

        if let Some(stripped) = &shortpath_name {
            let expanded_path = spaths.get(stripped).unwrap(); // Lookup the actual path
            output = output.replace(compstr, expanded_path.to_str().unwrap());
        }
        trace!("output: {}", output);
    }
    PathBuf::from(output)
}

fn find_key_for_value(spaths: &Shortpaths, value: PathBuf) -> Option<&String> {
    spaths.into_iter().find_map(|(k,v)| {
        if v.to_owned() == value {
            Some(k)
        } else {
            None
        }
    })
    //spaths.iter().find_map(|(key, val)| if val == value { Some(key) } else { None })
}

/// Folds nested shortpaths, environment variables
pub fn fold_shortpath(path: &Path, spaths: &Shortpaths) -> PathBuf {
    let mut output = path.to_str().unwrap().to_string();
    
    // Replace full exact matching paths with shortpath aliases
    //for (alias_name, alias_path) in spaths {
    for (_, alias_path) in spaths {
        //let path_string = alias_path.to_str().unwrap();
        let key = find_key_for_value(&spaths, alias_path.to_owned());
        //let alias = format!("${}", &alias_name); // Format with $alias_name

        match key {
            Some(name) => {
            //let alias = format!("${}", &key); // Format with $alias_name
            let alias = format!("${}", &name); // Format with $alias_name
            //output = output.replace(path_string, &alias);
            //output = output.replace(&alias, &alias);
            let replaced = alias_path.to_str().unwrap();
            output = output.replace(replaced, &alias);
            }
            None => {}
        }
    }
    let folded = PathBuf::from(output);
    folded
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
    trace!("Expanded shortpath: {}", expanded.display());

    let mut next = expanded.as_path();

    let mut new_path = PathBuf::new();
    //while next.parent().unwrap() != shortpath {
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
