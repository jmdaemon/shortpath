use std::{
    path::{Path, PathBuf},
    collections::HashMap,
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
pub fn find_matching_path(shortpath: &Path) -> PathBuf {
    let mut next = shortpath;

    let mut new_path = PathBuf::new();
    while next.parent().unwrap() != shortpath {
        // Check if the next directory contains any files that match our old path filename
        debug!("Getting list of files of directory {}", next.display());
        let parent_files = WalkDir::new(next).max_depth(1);
        
        // Check if any of these files match our given file name
        debug!("Searching for matching file names");
        let mut files: Vec<DirEntry> = vec![];
        for file in parent_files {
            if let Ok(e) = file {
                if has_equal_fname(&e, shortpath) {
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
