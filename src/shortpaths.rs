use crate::export::{Export, get_exporter};
use crate::helpers::{
    find_by_matching_path,
    find_paths,
    to_str_slice,
};
use std::{
    path::{Path, PathBuf, Component},
    cmp::Ordering,
};

use indexmap::IndexMap;
#[allow(unused_imports)]
use itertools::Itertools;
use serde::{Serialize, Serializer, Deserialize};
use walkdir::DirEntry;

// Data Types

pub type SP = IndexMap<String, Shortpath>;
pub type SPT = ShortpathType;
pub type SPD = ShortpathDependency;
pub type DEPS = Vec<SPD>; 

// Make invalid states inexpressible

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum ShortpathType {
    Path(String, PathBuf),      // Shortpath Name   : Shortpath Path
    AliasPath(String, PathBuf), // Shortpath Name   : Shortpath Path
    EnvPath(String, PathBuf),   // Env Var Name     : Shortpath Path
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortpathDependency {
    None,
    Shortpath(String),
    EnvironmentVariable(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortpath {
    pub path: SPT,
    #[serde(skip)]
    pub full_path: Option<PathBuf>,
    #[serde(skip)]
    pub deps: Option<DEPS>,
}

pub struct ShortpathsBuilder {
    paths: Option<SP>,
}

// Trait Implementations

// Serialize ShortpathType as &str
impl Serialize for ShortpathType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(get_shortpath_path(self).to_str().unwrap())
    }
}

// Sort paths in lexicographical order according to their full on disk paths
impl Ord for Shortpath {
    fn cmp(&self, other: &Self) -> Ordering {
        let (path_1, path_2) = (self.full_path.clone().unwrap(), other.full_path.clone().unwrap());
        path_1.cmp(&path_2)
    }
}

impl PartialOrd for Shortpath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Data Type Implementations

impl Shortpath {
    pub fn new(path: SPT, full_path: Option<PathBuf>, deps: Option<DEPS>) -> Shortpath {
        Shortpath { path, full_path, deps }
    }

    pub fn path(&self) -> &PathBuf {
        match &self.path {
            SPT::Path(_, path) | SPT::AliasPath(_, path) | SPT::EnvPath(_, path) => path
        }
    }

    pub fn name(&self) -> &String {
        match &self.path {
            SPT::Path(name, _) | SPT::AliasPath(name, _) | SPT::EnvPath(name, _) => name,
        }
    }
}

impl ShortpathsBuilder {
    // TODO: Use FromIterator trait extension
    pub fn new(sps: Vec<Shortpath>) -> ShortpathsBuilder  {
        let im = ShortpathsBuilder::from_vec(sps);
        ShortpathsBuilder { paths: Some(im) }
    }

    pub fn from_vec(sps: Vec<Shortpath>) -> SP {
        let mut im: SP = indexmap::IndexMap::new();
        sps.into_iter().for_each(|sp| {
            im.insert(sp.name().clone(), sp);
        });
        im
    }
    pub fn build(&mut self) -> Option<SP> {
        if let Some(shortpaths) = &mut self.paths {
            let shortpaths = populate_shortpaths(shortpaths);
            return Some(shortpaths);
        }
        None
    }
}

// Trait Extensions
// We need these for remove_hook, move_hook to check shortpath configs
pub trait FindKeyIndexMapExt<'a, K, V> {
    /// Get keys from value of IndexMap
    fn find_keys_for_value(&'a self, value: V) -> Vec<&'a K>;

    /// Get key from value of IndexMap
    fn find_key_for_value(&'a self, value: V) -> Option<&'a K>;
}

impl<'a, V> FindKeyIndexMapExt<'a, String, V> for IndexMap<String, Shortpath>
where
    V: Into<String>
{
    fn find_keys_for_value(&'a self, value: V) -> Vec<&'a String> {
        let v = value.into();
        self.into_iter()
            .filter_map(|(key, val)| if *val.path().to_str().unwrap() == v { Some(key) } else { None })
            .collect()
    }

    fn find_key_for_value(&'a self, value: V) -> Option<&'a String> {
        let v = value.into();
        self.iter().find_map(|(key, val)| if *val.path().to_str().unwrap() == v { Some(key) } else { None })
    }
}

// Pure Functions

pub fn expand_path(src: &str, key_name: &str, key_path: &str) -> String {
    let this = format!("${}", key_name);
    let with = key_path;
    src.replace(&this, with)
}

pub fn fold_path(src: &str, key_name: &str, key_path: &str) -> String {
    let this = key_path;
    let with = format!("${}", key_name);
    src.replace(this, &with)
}

pub fn get_shortpath_name(sp: &SPT) -> String {
    match sp {
        SPT::Path(name, _) | SPT::AliasPath(name, _) | SPT::EnvPath(name, _) => name.to_owned(),
    }
}

pub fn get_shortpath_dep_name(sp: &SPD) -> Option<String> {
    match sp {
        SPD::Shortpath(name) | SPD::EnvironmentVariable(name) => Some(name.to_owned()),
        SPD::None => None
        //SPT::Path(name, _) | SPT::AliasPath(name, _) | SPT::EnvPath(name, _) => name.to_owned(),
    }
}

pub fn get_shortpath_path(sp: &SPT) -> PathBuf {
    match sp {
        SPT::Path(_, path) | SPT::AliasPath(_, path) | SPT::EnvPath(_, path) => path.to_owned()
    }
}

pub fn sort_shortpaths(shortpaths: SP) -> SP {
    shortpaths.sorted_by(|_, v1, _, v2| {
        v1.cmp(v2)
    }).collect()
}

// Input Parsing

// Populate Shortpaths
pub fn populate_dependencies(shortpaths: &mut SP) -> SP {
    let shortpaths: SP = shortpaths.into_iter().filter_map(|(k, sp)| {
        let deps = find_deps(sp.path());
        sp.deps = Some(deps);
        Some((k.to_owned(), sp.to_owned()))
    }).collect();
    shortpaths
}

pub fn populate_expanded_paths(shortpaths: &mut SP) -> SP {
    let shortpaths_copy = shortpaths.clone();
    // Expand to full_path
    let shortpaths: SP = shortpaths.into_iter().filter_map(|(k, sp)| {
        let full_path = expand_shortpath(sp, &shortpaths_copy);
        sp.full_path = Some(full_path);
        Some((k.to_owned(), sp.to_owned()))
    }).collect();
    shortpaths
}

pub fn populate_shortpaths(shortpaths: &mut SP) -> SP {
    let mut shortpaths = populate_dependencies(shortpaths);
    populate_expanded_paths(&mut shortpaths)
}

/** Parse a Shortpath entry, and returns any dependencies */
pub fn parse_alias(path: &[char], full_path: PathBuf) -> Option<SPT> {
    match path {
        ['$', alias_name @ ..] => {
            let (an, ap) = (alias_name.iter().collect(), full_path);
            Some(SPT::AliasPath(an, ap))
        }
        [ '{', '$', 'e', 'n', 'v', ':', alias_name @ .., '}'] => {
            let (an, ap) = (alias_name.iter().collect(), full_path);
            Some(SPT::EnvPath(an, ap))
        }
        _ => { None }
    }
}

pub fn get_shortpath_dependency(path: &[char]) -> SPD {
    let to_string = |slice: &[char]| { slice.iter().collect() };
    let _env_prefix = to_str_slice("$env:");
    match path {
        ['$', alias_name @ ..]                  => SPD::Shortpath(to_string(alias_name)),
        [ _env_prefix, env_var_name @ .., '}']  => SPD::EnvironmentVariable(to_string(env_var_name)),
        _                                       => SPD::None
    }
}

pub fn get_shortpath_type(name: impl Into<String>, path: &PathBuf) -> SPT {
    let spt = match &path.to_str().unwrap().to_owned().to_lowercase() {
        keyword if keyword.contains('$')        => SPT::AliasPath(name.into(), path.to_owned()),
        keyword if keyword.contains("${env:")   => SPT::EnvPath(name.into(), path.to_owned()),
        _                                       => SPT::Path(name.into(), path.to_owned())
    };
    spt
}

/// Find the dependencies for a given shortpath
pub fn find_deps(entry: &Path) -> DEPS {
    let deps: DEPS = entry.components().into_iter().filter_map(|path_component| {
        if let Component::Normal(osstr_path) = path_component {
            let path_comp = to_str_slice(osstr_path.to_string_lossy());
            let dep = get_shortpath_dependency(&path_comp);
            return Some(dep);
        }
        None
    }).collect();
    deps
}

/**
  * Expand shortpath variants at runtime
  * 
  * The shortpath dependencies must be populated before this is run. When they are
  * populated, both their name and path values are stored in the enum variant which
  * is accessed here without the hashmap.
  */
pub fn expand_shortpath(sp: &Shortpath, shortpaths: &SP) -> PathBuf {
    let mut entry = sp.path().to_owned();
    match &sp.deps {
        Some(deps) => // Expand entry into full_path
            deps.iter().for_each(|dep| {
                // TODO: Wrap in a while loop later to parse additional paths
                if let Some(name) = get_shortpath_dep_name(dep) {
                    let dep_shortpath = shortpaths.get(&name).unwrap();
                    let path = get_shortpath_path(&dep_shortpath.path);

                    let output = expand_path(entry.to_str().unwrap(), &name, path.to_str().unwrap());
                    entry = PathBuf::from(output);
                }
            }),
            None => {}
    };
    entry
}



// Commands
pub fn add_shortpath(shortpaths: &mut SP, name: String, path: PathBuf) {
    let spt = match parse_alias(&to_str_slice(path.to_str().unwrap()), path.clone()) {
        Some(spt) => spt,
        None => SPT::Path(name.clone(), path)
    };
    let shortpath = Shortpath::new(spt, None, None);
    shortpaths.insert(name, shortpath);
}

pub fn remove_shortpath(shortpaths: &mut SP, current_name: &str) -> Option<Shortpath> {
    shortpaths.remove(current_name)
}

pub fn find_unreachable(shortpaths: &SP) -> IndexMap<&String, &Shortpath> {
    let unreachable: IndexMap<&String, &Shortpath> = shortpaths.iter()
        .filter(|(_, path)| { !path.path().exists() || path.path().to_str().is_none() }).collect();
    unreachable
}

/// List any broken or unreachable paths
pub fn check_shortpaths(shortpaths: &mut SP) {
    let unreachable = find_unreachable(shortpaths);
    unreachable.iter().for_each(|(alias_name, alias_path)|
        println!("{} shortpath is unreachable: {}", alias_name, alias_path.path().display()));
    println!("Check Complete");
}

/** Fix unreachable or broken paths
  * 
  * There are a few different resolve types to select from:
  * - Exact Matching Path in Parent directory
  * - Exact Matching Path in Nearest Neighbours
  * - Most Similar Names in Parent Directory
  * - Most Similar Names in Nearest Neighbours
  * - Find names using locatedb
  *
  * In addition there are also a few options:
  * - Automode: Automatically selects and updates the best candidate shortpath, given the selected resolve_type
  * - Manual: The user is given the option to: (overwrite, overwrite_all, skip, skipall)
  *
  * TODO: How to implement additional search modes?
  * TODO: When implementing matching by nearest neighbours, think about how to 
  * encode the scope as a function parameter
  * TODO: Add overwrite_all, skip_all flags
  * TODO: Create a data structure for the flags
  */
pub fn resolve(shortpaths: &mut SP, resolve_type: &str, automode: bool) {
    // Automode: Make the decision for the user
    let automode_fn = |sp: &Shortpath, results: Vec<DirEntry>| {
        let first = results.first().unwrap();
        let (name, path) = (sp.name().to_owned(), first.path().to_owned());
        (name, path)
    };

    // Manual: Provide options at runtime for the user
    let manualmode_fn = |sp: &Shortpath, _results: Vec<DirEntry>| {
        let (name, path) = (sp.name(), sp.path().to_owned());
        // TODO Wait for the user to make a decision
        println!("Not yet implemented"); // TODO
        (name.to_owned(), path)
    };

    // Feature Selection Closures
    let find_by = match resolve_type {
        "matching" => find_by_matching_path,
        _ => find_by_matching_path,
    };

    let resolve_fn = match automode {
        true => automode_fn,
        false => manualmode_fn, // We don't have a proper implementation yet for the other one
    };

    let updates: Vec<(String, PathBuf)> = shortpaths.iter().filter_map(|(_,sp)| {
        if let Some(results) = find_paths(sp, find_by) {
            let current_path = sp.path();
            let (name, path) = resolve_fn(sp, results);

            if &path != current_path {
                println!("Updating shortpath {} from {} to {}", name, current_path.display(), path.display());
            } else {
                println!("Keeping shortpath {}: {}", name, path.display());
            }
            Some((name, path))
        } else {
            None
        }
    }).collect();
    
    // Perform the update
    updates.into_iter().for_each(|(name, path)| {
        update_shortpath(shortpaths, &name, None, Some(&path.to_str().unwrap().to_owned()));
    });
}

/** Serialize shortpaths to other formats for use in other applications */
pub fn export_shortpaths(shortpaths: &SP, export_type: &str, output_file: Option<&String>) -> PathBuf {
    let exp = get_exporter(export_type)
        .set_shortpaths(shortpaths);
    let dest = exp.prepare_directory(output_file);
    exp.write_completions(dest)
}

/** Update a single shortpath's alias name or path
  * Changes the name or path if given and are unique */
pub fn update_shortpath(shortpaths: &mut SP, current_name: &str, name: Option<&String>, path: Option<&String>) {
    let update_name = |new_path: String, shortpaths: &mut SP| {
        let path = PathBuf::from(new_path);
        
        let spt = match parse_alias(&to_str_slice(path.to_str().unwrap()), path.clone()) {
            Some(spt) => spt,
            None => SPT::Path(current_name.to_owned(), path)
        };
        let shortpath = Shortpath::new(spt, None, None);
        shortpaths.insert(current_name.to_owned(), shortpath);
    };
    let update_path = |new_name: String, shortpaths: &mut SP| {
        let path = shortpaths.remove(current_name).unwrap();
        shortpaths.insert(new_name, path);
    };

    match (name, path) {
        (Some(new_name), _) => { update_name(new_name.to_owned(), shortpaths); }
        (_, Some(new_path)) => { update_path(new_path.to_owned(), shortpaths); }
        (_, _)              => { println!("Nothing to do");}
    }
}
