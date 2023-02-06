use crate::shortpaths::Shortpath;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use log::{debug, trace};
use walkdir::{DirEntry, WalkDir};

/// Convert strings into a vector of characters
pub fn to_str_slice(s: impl Into<String>) -> Vec<char> {
    s.into().chars().collect()
}

/// Find the longest key name in any IndexMap
pub fn find_longest_keyname<T>(map: IndexMap<String, T>) -> String {
    map.iter()
       .max_by(|(k1,_), (k2, _)| k1.len().cmp(&k2.len()))
       .unwrap().0.to_owned()
}

/// Expands ~/ to the user's home
pub fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Option<PathBuf> {
    let p = path_user_input.as_ref();
    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }
    if p == Path::new("~") {
        return dirs::home_dir();
    }
    dirs::home_dir().map(|mut h| {
        if h == Path::new("/") {
            // Corner case: `h` root directory;
            // don't prepend extra `/`, just drop the tilde.
            p.strip_prefix("~").unwrap().to_path_buf()
        } else {
            h.push(p.strip_prefix("~/").unwrap());
            h
        }
    })
}

/// Attempt to find the a file in a dir
pub fn find_by_matching_path(file_name: &str, dir: WalkDir) -> Vec<DirEntry> {
    let files: Vec<DirEntry> = dir.into_iter()
        .filter_map(Result::ok)
        .filter(|file| file.file_name() == file_name)
        .collect();
    files
}

pub fn find_paths(sp: &Shortpath, find_by: impl Fn(&str, WalkDir) -> Vec<DirEntry>) -> Option<Vec<DirEntry>> {
    let search_term = sp.path().file_name().unwrap();
    let mut next = sp.path().parent();
    
    while let Some(dir) = next {
        debug!("In Directory {}", dir.display());
        let parent_files = WalkDir::new(dir).max_depth(1);

        debug!("Searching for files");
        let files = find_by(search_term.to_str().unwrap(), parent_files);
        files.iter().for_each(|f| trace!("File: {}", f.file_name().to_str().unwrap()));

        if files.is_empty() {
            return Some(files);
        }
        next = dir.parent(); // Continue searching
    }
    None
}

// Create function to format shortpaths with spaces
pub fn tab_align(s: &str, delim: &str) -> String {
    let length = s.len();
    //let args = format!("{: <length$}{delim}", s, delim);
    let args = format!("{: <length$}{delim}", s);
    //let args = format!("{: <length$} =", s);
    //let args = format!("{: <length$} =", s);
    //format!("{}: {}", format_args!("{} {}", s, args), s)
    //format!("{}{}\n", args, s)
    args
}
