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
use log::trace;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use walkdir::DirEntry;

// Data Types
pub type SP = IndexMap<String, Shortpath>;
pub type SPD = ShortpathDependency;
pub type DEPS = Vec<SPD>; 

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortpathVariant {
    Independent,
    Alias,
    Environment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortpathDependency(ShortpathVariant, String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortpath {
    pub path: PathBuf,
    pub full_path: Option<PathBuf>,
    pub deps: DEPS,
}

pub struct ShortpathsBuilder {
    paths: Option<SP>,
}

// Trait Implementations

// Serialize Shortpath as &str
impl Serialize for Shortpath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.path.to_str().unwrap())
    }
}

// Parse &str into Shortpath
impl<'de> Deserialize<'de> for Shortpath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path: &str = Deserialize::deserialize(deserializer)?;
        let sp = Shortpath::new(PathBuf::from(path), None, vec![]);
        Ok(sp)
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
    pub fn new(path: PathBuf, full_path: Option<PathBuf>, deps: DEPS) -> Shortpath {
        Shortpath { path, full_path, deps }
    }
}

impl ShortpathsBuilder {
    // TODO: Use FromIterator trait extension
    pub fn new(sp: SP) -> ShortpathsBuilder  {
        ShortpathsBuilder { paths: Some(sp) }
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
            .filter_map(|(key, val)| if val.path.to_str().unwrap() == v { Some(key) } else { None })
            .collect()
    }

    fn find_key_for_value(&'a self, value: V) -> Option<&'a String> {
        let v = value.into();
        self.iter().find_map(|(key, val)| if val.path.to_str().unwrap() == v { Some(key) } else { None })
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

pub fn sort_shortpaths(shortpaths: SP) -> SP {
    shortpaths.sorted_by(|_, v1, _, v2| {
        v1.cmp(v2)
    }).collect()
}

// Input Parsing

// Populate Shortpaths
pub fn populate_dependencies(shortpaths: &mut SP) -> SP {
    let c = shortpaths.clone();
    let shortpaths: SP = shortpaths.into_iter().map(|(k, sp)| {
        let deps = find_deps(&sp.path, false, &c);
        trace!("Dependencies for Shortpath: {} : {:?}", k, deps);
        sp.deps = deps;
        (k.to_owned(), sp.to_owned())
    }).collect();
    shortpaths
}

pub fn populate_expanded_paths(shortpaths: &mut SP) -> SP {
    let shortpaths_copy = shortpaths.clone();
    // Expand to full_path
    let shortpaths: SP = shortpaths.into_iter().map(|(k, sp)| {
        let full_path = expand_shortpath(sp, &shortpaths_copy);
        sp.full_path = Some(full_path);
        (k.to_owned(), sp.to_owned())
    }).collect();
    shortpaths
}

pub fn populate_shortpaths(shortpaths: &mut SP) -> SP {
    let mut shortpaths = populate_dependencies(shortpaths);
    populate_expanded_paths(&mut shortpaths)
}

/** Parse a Shortpath entry, and returns any dependencies */
pub fn get_shortpath_dependency(path: &[char]) -> Option<SPD> {
//pub fn get_shortpath_dependency(path: &[char]) -> SPD {
    // Closures
    let to_string = |slice: &[char]| { slice.iter().collect() };
    let _env_prefix = to_str_slice("$env:");
    let is_not_empty = |path: &[char]| { !path.is_empty() };
    let to_spd = |variant: ShortpathVariant, dependency_name: String| { ShortpathDependency(variant, dependency_name) };

    match (is_not_empty(path), path) {
        (true, ['$', alias_name @ ..])                  => Some(to_spd(ShortpathVariant::Alias, to_string(alias_name))),
        (true, [ _env_prefix, env_var_name @ .., '}'])  => Some(to_spd(ShortpathVariant::Environment, to_string(env_var_name))),
        (true, _)                                       => Some(to_spd(ShortpathVariant::Independent, to_string(path))),
        (false, _)                                      => None
    }
}

/** Find the dependencies for a given shortpath
  *
  * Algorithm:
  * For every component in the path.components()
  *     If component is a valid shortpath dependency, add it to our array.
  *
  * For every shortpath dependency in our array
  *     If any shortpath dependency is another shortpath dependency
  *         Recursively yield the next dependency, and add it to our array
  * At the end, merge the two vectors together into one vector */
pub fn find_deps(entry: &Path, in_find_nested_mode: bool, shortpaths: &SP) -> DEPS {
    let mut dependencies: Vec<ShortpathDependency> = Vec::new();

    // For every component in the path.components()
    for comp in entry.components() {
        trace!("entry: {:#?}", entry.display());
        trace!("comp: {:?}", comp);
        if let Component::Normal(osstr_path) = comp {
            // If component is a valid shortpath dependency
            let path_comp_slice = to_str_slice(osstr_path.to_string_lossy());
            let dep = get_shortpath_dependency(&path_comp_slice);

            match dep {
                Some(dep_variant) => {
                    // Add it to our array.
                    //if in_find_nested_mode == false {
                    if !in_find_nested_mode {
                        dependencies.push(dep_variant.clone());
                    }
                    trace!("dependencies: {:?}", &dependencies);
                    match dep_variant.0 {
                        ShortpathVariant::Independent => { }
                        ShortpathVariant::Alias => {
                            dependencies.push(dep_variant.clone());
                        }
                        ShortpathVariant::Environment => {
                            dependencies.push(dep_variant.clone());
                        }
                    };
                }
                None => {
                    // Otherwise, end our loop
                    break;
                }
            }
        }
    }
    if dependencies.is_empty() {
        return dependencies;
    }

    // Nested Dependencies
    let is_nested_dep = |dep: &ShortpathDependency, shortpaths: &SP| {
        shortpaths.get(&dep.1).is_some()
    };

    let mut nested_deps: Vec<ShortpathDependency> = Vec::new();

    // For every shortpath dependency in our array
    for dep in dependencies.clone() {
        // If not a nested dependency
        if !is_nested_dep(&dep, shortpaths) {
            break; // Exit
        }

        //if in_find_nested_mode == false {
        //if !in_find_nested_mode {
            //in_find_nested_mode = true;
        //}

        // Recursively yield the next dependency
        let nested_dep = shortpaths.get(&dep.1).unwrap();
        trace!("nested_dep: {}", nested_dep.path.display());
        let entry = &nested_dep.path;
        nested_deps.append(&mut find_deps(entry, true, shortpaths));
    }
    dependencies.append(&mut nested_deps); // Flatten
    trace!("dependencies: {:?}", dependencies);
    dependencies
}

/**
  * Expand shortpath variants at runtime
  * 
  * The shortpath dependencies must be populated before this is run. When they are
  * populated, both their name and path values are stored in the enum variant which
  * is accessed here without the hashmap.
  */
pub fn expand_shortpath(sp: &Shortpath, shortpaths: &SP) -> PathBuf {
    let mut entry = sp.path.to_owned();
    let deps = &sp.deps;
    deps.iter().for_each(|dep| {
        // TODO: Wrap in a while loop later to parse additional paths
        let name = &dep.1;
        if let Some(dep_shortpath) = shortpaths.get(name) {
            let path = dep_shortpath.path.to_owned();

            let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
            entry = PathBuf::from(output);
        }
    });
    entry
}

// Commands
pub fn add_shortpath(shortpaths: &mut SP, name: String, path: PathBuf) {
    let shortpath = Shortpath::new(path, None, vec![]);
    shortpaths.insert(name, shortpath);
}

pub fn remove_shortpath(shortpaths: &mut SP, current_name: &str) -> Option<Shortpath> {
    shortpaths.remove(current_name)
}

pub fn find_unreachable(shortpaths: &SP) -> IndexMap<&String, &Shortpath> {
    let unreachable: IndexMap<&String, &Shortpath> = shortpaths.iter()
        .filter(|(_, path)| { !path.path.exists() || path.path.to_str().is_none() }).collect();
    unreachable
}

/// List any broken or unreachable paths
pub fn check_shortpaths(shortpaths: &mut SP) {
    let unreachable = find_unreachable(shortpaths);
    unreachable.iter().for_each(|(alias_name, alias_path)|
        println!("{} shortpath is unreachable: {}", alias_name, alias_path.path.display()));
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
    let automode_fn = |shortpaths: &SP, sp: &Shortpath, results: Vec<DirEntry>| {
        let first = results.first().unwrap();
        let path = first.path().to_owned();
        let name = shortpaths.find_key_for_value(path.to_str().unwrap()).unwrap();
        (name.to_owned(), path)
    };

    // Manual: Provide options at runtime for the user
    let manualmode_fn = |shortpaths: &SP, sp: &Shortpath, results: Vec<DirEntry>| {
        let path = sp.path.to_owned();
        let name = shortpaths.find_key_for_value(path.to_str().unwrap()).unwrap();
        //let (name, path) = (sp.name.to_owned(), sp.path.to_owned());
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
            let current_path = sp.path.to_owned();
            let (name, path) = resolve_fn(shortpaths, sp, results);

            if path != current_path {
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
    let entry_exists = || { shortpaths.get(current_name).is_some() }; 

    let update_path = |new_path: String, shortpaths: &mut SP| {
        let shortpath = Shortpath::new(PathBuf::from(new_path), None, vec![]);
        shortpaths.insert(current_name.to_owned(), shortpath);
    };
    let update_name = |new_name: String, shortpaths: &mut SP| {
        let path = shortpaths.remove(current_name).unwrap();
        shortpaths.insert(new_name, path);
    };

    match (entry_exists(), name, path) {
        (true, Some(new_name), _) => { update_name(new_name.to_owned(), shortpaths); }
        (true, _, Some(new_path)) => { update_path(new_path.to_owned(), shortpaths); }
        (_, _, _)              => { println!("Nothing to do");}
    }
}
