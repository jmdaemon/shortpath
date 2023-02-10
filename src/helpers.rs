use crate::shortpaths::{Shortpath, SP};

use std::{
    env::var,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use log::{debug, trace};
use walkdir::{DirEntry, WalkDir};

/// Convert strings into a vector of characters
pub fn to_str_slice(s: impl Into<String>) -> Vec<char> {
    s.into().chars().collect()
}

/// Find the longest key name in any IndexMap
pub fn find_longest_keyname<T>(map: &IndexMap<String, T>) -> String {
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

// Resolve predicates
//pub fn is_matching(name: &str, entry: DirEntry) -> bool {
    //entry.file_name() == name
//}

//pub fn levehstein() { }

//pub fn is_similar_levehstein() {
//}

// Resolve traits
//trait ResolveDirFindExt {
    //fn find_by_matching_name(&self, name: &str) -> Vec<DirEntry>;
    ////fn find_by_similar_name_levehstein(&self, name: &str) -> Vec<DirEntry>;
//}

//pub trait ShortpathsFindExt {
    //fn find_by(resolve_fn: impl Fn(&SP) -> Vec<DirEntry>) -> Vec<DirEntry>;
//}

//impl ShortpathsFindExt for SP {
    //fn find_by(resolve_fn: impl Fn(&SP) -> Vec<DirEntry>) -> Vec<DirEntry>;
//}

///// Attempt to find the a file in a dir
//pub fn find_by_matching_path(file_name: &str, dir: WalkDir) -> Vec<DirEntry> {
//    let files: Vec<DirEntry> = dir.into_iter()
//        .filter_map(Result::ok)
//        .filter(|file| file.file_name() == file_name)
//        .collect();
//    files
//}

//pub fn is_matching(name: &str, dir: DirEntry) -> bool {
    //entry.file_name() == name
//}


//pub fn find_matching_name_in_dir(sp: &Shortpath, dir: WalkDir) -> Vec<DirEntry> {
    ////let search_for = sp.path.file_name().unwrap().to_str().unwrap();
    ////let search_for = sp.path.file_name().unwrap().to_string_lossy();
    //let file_name = sp.path.file_name().unwrap().to_str().unwrap();
    //dir.into_iter()
        //.filter_map(Result::ok)
        //.filter(|file| file.file_name() == file_name)
        //.collect()
////    files
//}

//pub fn find_paths(sp: &Shortpath, find_by: impl Fn(&str, WalkDir) -> Vec<DirEntry>) -> Option<Vec<DirEntry>> {
//pub fn find_by_matching_path(shortpaths: &SP) -> Vec<DirEntry> {
//pub fn search_parent_dir(sp: &Shortpath, search_fn: impl Fn (&Shortpath, WalkDir) -> Vec<DirEntry>) -> Vec<DirEntry> {
    //let search_term = sp.path.file_name().unwrap();
    //let mut next = sp.path.parent();
    
    //let mut files = vec![];
    //while let Some(dir) = next {
        //debug!("In Directory {}", dir.display());
        //let parent_files = WalkDir::new(dir).max_depth(1);

        //debug!("Searching for files");
        ////let files = find_by(search_term.to_str().unwrap(), parent_files);
        //files = search_fn(sp, parent_files);
        //files.iter().for_each(|f| trace!("File: {}", f.file_name().to_str().unwrap()));

        //if files.is_empty() {
            //return files;
        //}
        //next = dir.parent(); // Continue searching
    //}
    //return files;
//}


//pub fn find_by_matching_names(shortpaths: &SP) -> Vec<DirEntry> {
//pub fn search_for(shortpaths: &SP) -> Vec<DirEntry> {
pub type SearchResults = Vec<DirEntry>;
pub type ScopeResults = Vec<(PathBuf, SearchResults)>;
pub type SearchFn = fn(&Shortpath, WalkDir) -> SearchResults;
//type ScopeFn = fn(SearchFn) -> SearchResults;
pub type ScopeFn = fn(&Shortpath, SearchFn) -> ScopeResults;

// Scope Functions

/// Search for files in parent directories
/// Returns the first set of matching results
/// NOTE: This may be adjusted later to return more than just the first set of matching results
/// if it is fast and efficient enough, for use in more complex functions
//pub fn in_parent_dir(sp: &Shortpath, search_fn: SearchFn) -> SearchResults {
//pub fn in_parent_dir(sp: &Shortpath, search_fn: SearchFn) -> Vec<SearchResults> {
pub fn in_parent_dir(sp: &Shortpath, search_fn: SearchFn) -> ScopeResults {
    let full_path = sp.full_path.clone().unwrap();
    let mut next = full_path.parent();
    
    let mut found = vec![];
    while let Some(dir) = next {
        debug!("Searching Directory {}", dir.display());
        let parent_files = WalkDir::new(dir).max_depth(1);

        let mut matches = search_fn(sp, parent_files);
        matches.iter().for_each(|f| trace!("Found: {}", &f.file_name().to_str().unwrap()));
        //found.append(&mut matches);
        //found.push(&mut matches);
        found.push((dir.to_path_buf(), matches));

        next = dir.parent(); // Continue searching
    }
    found
}

// Search Functions
pub fn matching_file_names(sp: &Shortpath, dir: WalkDir) -> Vec<DirEntry> {
    let file_name = sp.path.file_name().unwrap().to_str().unwrap();
    dir.into_iter()
        .filter_map(Result::ok)
        .filter(|file| file.file_name() == file_name)
        .collect()
}

// Resolve Mode

/// Automatically chooses the best candidate to resolve the shortpath to
pub fn auto_resolve(results: SearchResults) -> Option<DirEntry> {
    Some(results.first().unwrap().to_owned())
    //if let Some(entry) = results.first() {
        //entry
    //}
    //results.as_ref
    //Some(results.first().map(|f| f.to_owned()))
}

/// Manually prompts user to
pub fn manual_resolve(results: SearchResults, shortpaths: &mut SP, unreachable: &SP) -> Option<DirEntry> {
    None
    //for 
    //results.iter().for_each(|file| {
        //println!("Update {}");
    //});
}

//pub fn search_for(search_fn: SearchFn, scope_fn: ScopeFn, shortpaths: &SP) -> Vec<SearchResults> {
//pub fn search_for(search_fn: SearchFn, scope_fn: ScopeFn, shortpaths: &SP) -> IndexMap<String, Vec<SearchResults>> {
//pub fn search_for(search_fn: SearchFn, scope_fn: ScopeFn, shortpaths: &SP) -> IndexMap<String, Vec<SearchResults>> {
pub fn search_for(search_fn: SearchFn, scope_fn: ScopeFn, shortpaths: &SP) -> IndexMap<String, ScopeResults> {
    //let search_results = vec![];
    let results = shortpaths.iter().map(|(name, sp)| {
        let found = scope_fn(sp, search_fn);
        //search_results.push(found);
        //(name.to_owned(), sp.to_owned())
        //(name.to_owned(), sp.to_owned())
        (name.to_owned(), found)
    }).collect();
    results
    /*
    let mut results = vec![];
    shortpaths.iter().for_each(|(_, sp)| {
        //let asdf = search_fn;
        let found = scope_fn(sp, search_fn);
        results.push(found);
    });
    results
    */
}

/** Tab align right strings
 * NOTE: Add option to change tab direction
 **/
pub fn tab_align(s: &str, width: usize, delim: Option<&str>) -> String {
    if let Some(delim) = delim {
        format!("{: <width$}{}", s, delim)
    } else {
        format!("{: <width$} ", s)
    }
}

/// Read environment variable to String
pub fn getenv<S: Into<String>>(name: S) -> String {
    let name = name.into();
    let mut path = String::new();
    match var(&name) {
        Ok(val) => path = val,
        Err(e) => eprintln!("Error in expanding environment variable ${}: ${}", name, e)
    };
    path
}

/// Sort shortpaths according to the lexicographical order of their expanded paths
/// Note that this returns a copy of the shortpaths
pub fn sort_shortpaths(shortpaths: SP) -> SP {
    shortpaths.sorted_by(|_, v1, _, v2| {
        v1.cmp(v2)
    }).collect()
}
