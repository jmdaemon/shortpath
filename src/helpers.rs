use crate::shortpaths::{Shortpath, SP};
use std::{
    env::var,
    path::{Path, PathBuf},
    io::{stdin, stdout, Write},
};

use indexmap::IndexMap;
use log::{debug, trace, info};
use walkdir::{DirEntry, WalkDir};

// Types
pub type SearchResults = Vec<DirEntry>;
pub type ScopeResults = Vec<(PathBuf, SearchResults)>;
pub type SearchFn = fn(&Shortpath, WalkDir) -> SearchResults;
pub type ScopeFn = fn(&Shortpath, SearchFn) -> ScopeResults;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ResolveChoices {
    Overwrite,
    OverwriteAll,
    Skip,
    SkipAll,
}

// Helper Functions

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

/// Get user input
pub fn prompt(message: &str) -> Option<String> {
    info!("prompt()");
    print!("{}", message);
    stdout().lock().flush().expect("Unable to write prompt to STDOUT");

    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to get user input");
    debug!("Input Received: {}", &input.trim_end());

    if !input.is_empty() { Some(input) } else { None }
}

/// Repeatedly prompt user until a valid input is given
pub fn prompt_until_valid(message: &str, is_valid: impl Fn(String) -> bool) -> String {
    let mut input = prompt(message);
    while !is_valid(input.clone().unwrap()) {
        input = prompt(message);
    }
    input.unwrap()
}

/// Tab align right strings
/// NOTE: Add option to change tab direction
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

/// Returns a sorted copy of the shortpaths
pub fn sort_shortpaths(shortpaths: SP) -> SP {
    shortpaths.sorted_by(|_, v1, _, v2| {
        v1.cmp(v2)
    }).collect()
}

// Searching

/// Find files for unreachable shortpaths
pub fn search_for(search_fn: SearchFn, scope_fn: ScopeFn, unreachable: &SP) -> IndexMap<String, ScopeResults> {
    info!("search_for()");
    unreachable.iter().map(|(name, sp)| {
        let found = scope_fn(sp, search_fn);
        (name.to_owned(), found)
    }).collect()
}


// Scope Functions

/// Search for files in parent directories
/// Returns the first set of matching results
/// NOTE: This may be adjusted later to return more than just the first set of matching results
/// if it is fast and efficient enough, for use in more complex functions
pub fn in_parent_dir(sp: &Shortpath, search_fn: SearchFn) -> ScopeResults {
    let full_path = sp.full_path.clone().unwrap();
    let mut next = full_path.parent();
    
    let mut found = vec![];
    while let Some(dir) = next {
        debug!("Searching Directory {}", dir.display());
        let parent_files = WalkDir::new(dir).max_depth(1);
        let matches = search_fn(sp, parent_files);
        matches.iter().for_each(|f| trace!("\tFound: {}", &f.file_name().to_str().unwrap()));
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
pub fn auto_resolve(name: String, results: ScopeResults) -> Option<(String, PathBuf)> {
    let results: Vec<Vec<DirEntry>> = results.into_iter()
        .filter(|(_, nested_entries)| !nested_entries.to_owned().is_empty())
        .into_iter()
        .map(|(_,nested_entries)| nested_entries)
        .collect();

    if !results.is_empty() {
        let path = results.first().unwrap().first().unwrap().to_owned();
        return Some((name, path.path().to_path_buf()))
    }
    None
}

// Manual Resolve

pub fn get_choice(input: String) -> Option<ResolveChoices> {
    match input.to_lowercase().as_str().trim_end() {
        "overwrite"     => Some(ResolveChoices::Overwrite),
        "overwrite_all" => Some(ResolveChoices::OverwriteAll),
        "skip"          => Some(ResolveChoices::Skip),
        "skip_all"      => Some(ResolveChoices::SkipAll),
        _               => None
    }
}

/// Manually prompt user to resolve unreachable Shortpath
pub fn manual_resolve(name: String, previous: &Path, results: ScopeResults) -> Option<(String, PathBuf)> {
    let mut choice: Option<ResolveChoices> = None;
    let is_valid_input = |input: String| {
        matches!(input.to_lowercase().trim_end(), "overwrite" | "overwrite_all" | "skip" | "skip_all")
    };
    for (_, search_results) in results.iter() {
        for file in search_results {

            if (choice.is_some()) && (choice.as_ref() == Some(&ResolveChoices::OverwriteAll)) {
                return Some((name, file.path().to_path_buf()))
            }

            let message = format!("Update {} from {} to {}? [overwrite, overwrite_all, skip, skip_all]: ",
    name, &previous.display(), &file.path().display());
            let input = prompt_until_valid(&message, is_valid_input);
            choice = get_choice(input);

            match choice.unwrap() {
                ResolveChoices::Skip    => continue,
                ResolveChoices::SkipAll => continue,
                _                       => return Some((name, file.path().to_path_buf())),
            }
        }
    }
    None
}
