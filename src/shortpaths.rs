use crate::export::{Export, get_exporter};
use crate::helpers::{
    find_by_matching_path,
    find_paths,
    to_str_slice,
};
use std::{
    path::{PathBuf, Component},
    cmp::Ordering,
};

use indexmap::IndexMap;
#[allow(unused_imports)]
use itertools::Itertools;
use log::{trace, debug};
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
        let sp = Shortpath::new(PathBuf::from(path), None);
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
    pub fn new(path: PathBuf, full_path: Option<PathBuf>) -> Shortpath {
        Shortpath { path, full_path }
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
//pub fn populate_dependencies(shortpaths: &mut SP) -> SP {
    //let c = shortpaths.clone();
    //let shortpaths: SP = shortpaths.into_iter().map(|(k, sp)| {
        //let deps = find_deps(&sp.path, false, &c);
        //trace!("Dependencies for Shortpath: {} : {:?}", k, deps);
        //sp.deps = deps;
        //(k.to_owned(), sp.to_owned())
    //}).collect();
    //shortpaths
//}

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
    //let mut shortpaths = populate_dependencies(shortpaths);
    populate_expanded_paths(shortpaths)
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
        (true, [ normal_path @ ..])                     => Some(to_spd(ShortpathVariant::Independent, to_string(normal_path))),
        (false, _)                                      => None
    }
}

/** Return the type of a shortpath entry */
//pub fn get_shortpath_type(path: &[char]) -> Option<ShortpathVariant> {
//pub fn get_shortpath_type(comp: Component) -> Option<ShortpathVariant> {
pub fn get_shortpath_type(comp: &[char]) -> Option<ShortpathVariant> {
    //let to_string = |slice: &[char]| { slice.iter().collect() };
    let _env_prefix = to_str_slice("$env:");
    let is_not_empty = |path: &[char]| { !path.is_empty() };
    //let to_spd = |variant: ShortpathVariant, dependency_name: String| { ShortpathDependency(variant, dependency_name) };

    
    //comp.as_os_str().to_str().unwrap().to_string().chars())
    //match (is_not_empty(path), path) {
    //let is_not_empty = |comp: Component| { !comp.as_os_str().is_empty() };
    //let path = to_str_slice(&comp.as_os_str().to_str().unwrap();
    //match (is_not_empty(), &path) {
    match (is_not_empty(comp), comp) {
        (true, ['$', alias_name @ ..])                  => Some(ShortpathVariant::Alias),
        (true, [ _env_prefix, env_var_name @ .., '}'])  => Some(ShortpathVariant::Environment),
        (true, [ normal_path @ ..])                     => Some(ShortpathVariant::Independent),
        (false, _)                                      => None
    }
}
// Parses the alias, and only alias variant of a string
pub fn parse_alias(comp: &str) -> Option<String> {
    let copy = comp.to_string();
    if copy.starts_with('$') {
        Some(copy.split_at(1).1.to_owned())
        //copy.chars()
    } else {
        None
    }
}

pub fn to_string(comp: &Component) -> String {
    comp.as_os_str().to_str().unwrap().to_string()
}

pub fn str_join_path(s1: &str, s2: &str) -> PathBuf {
    let (p1,p2) = (PathBuf::from(s1), PathBuf::from(s2));
    p1.join(p2)
}

/**
  * Expand shortpath variants at runtime
  * 
  * The shortpath dependencies must be populated before this is run. When they are
  * populated, both their name and path values are stored in the enum variant which
  * is accessed here without the hashmap.
  */
pub fn expand_shortpath(sp: &Shortpath, shortpaths: &SP) -> PathBuf {
    //let mut entry = sp.path.to_owned();

    // Algorithm Example:
    // (alias_path='$c/dddd', entry='$c/dddd', sp)
    // -> $c
    //    -> alias
    //    -> '$b/cccc'
    //    -> ('$b/cccc', '$b/cccc', sp)
    //       -> ('$b/cccc', '$b/cccc', sp)
    //       -> $b
    //       -> alias
    //       -> '$a/bbbb'
    //       -> ('$a/bbbb', '$a/bbbb', sp)
    //          -> $a
    //          -> alias
    //          -> 'aaaa'
    //          -> ('aaaa', 'aaaa', sp)
    //             -> None 
    //             -> Return 'aaaa'
    //          -> Return expand_path(entry='$a/bbbb', alias_name='a', alias_path='aaaa')
    //       -> Return expand_path(entry='$b/cccc', alias_name='b', alias_path='aaaa/bbbb')
    //    -> Return expand_path(entry='$c/dddd', alias_name='c', alias_path='aaaa/bbbb/cccc')
    // -> Store entry, send as PathBuf
    //pub fn f(mut alias_path: String, entry: PathBuf, alias_name, shortpaths: &SP) -> Option<String> {

    // (String::new(), '$c/dddd', String::new(), sp)
    // -> ('$b=cccc', '$c/dddd', "$c", sp)
    //    -> ('$a/bbbb', '$b/dddd', "$a", sp)
    //       -> ('aaaa', '$a/bbbb', "", sp)
    //pub fn f(alias_name: Either<String, Option<String>>, alias_path: String, entry: PathBuf, shortpaths: &SP) -> String {
    pub fn f(alias_name: String, alias_path: String, entry: PathBuf, has_started: bool, shortpaths: &SP) -> String {
        println!();
        println!("Summary");
        dbg!(&alias_path);
        dbg!(&entry);
        dbg!(&alias_name);

        let mut output = String::new();

        if alias_name.is_empty() && has_started {
            return alias_path;
        }

        // Return alias path if entry has nothing
        if entry.components().peekable().peek().is_none() {
            return alias_path;
        }
        // Assume we can obtain a component
        let comp = entry.components().next().unwrap();
        let comp_slice = to_str_slice(comp.as_os_str().to_str().unwrap());
        let shortpath_type = get_shortpath_type(&comp_slice);

        // If we have nothing, exit early
        if shortpath_type.is_none() {
            return alias_path;
        }

        // Assume we can obtain a variant
        let shortpath_variant = shortpath_type.unwrap();

        match shortpath_variant {
            ShortpathVariant::Environment | ShortpathVariant::Independent => {
                return alias_path;
            }
            _ => {}
        };

        if ShortpathVariant::Alias == shortpath_variant {
            if !has_started {
                trace!("Beginning recursive expansion");
                trace!("Branch 1");

                trace!("Setting Name");
                let sp_depend_name = parse_alias(&to_string(&comp)).unwrap();
                let sp_depend_path = shortpaths.get(&sp_depend_name).unwrap();

                let depend_path = sp_depend_path.path.to_str().unwrap().to_string();
                debug!("depend_path = {}", &depend_path);
                assert_ne!(String::new(), depend_path);

                trace!("Starting recursion");

                let expanded = expand_path(entry.to_str().unwrap(), &sp_depend_name, &depend_path);
                trace!("Expanding layer: {} -> {}", entry.display(), &expanded);

                //output = expanded.clone();
                output = expanded;

                trace!("Expanding all layers...");
                let verbatim_depend_name = format!("${}", sp_depend_name);
                let result = f(verbatim_depend_name, depend_path, PathBuf::from(output), true, shortpaths);
                trace!("B1: Received result: {}", result);
                trace!("");
                output = result.clone();
                return result
            } else {
                trace!("In recursive expansion");
                trace!("Branch 2");

                if let Some(parsed) = parse_alias(&alias_name) {
                    trace!("Parsed alias_name: {}", parsed);

                    // Note that this only works for the first component of a string
                    let pbuf = PathBuf::from(alias_path);
                    let sp_depend_name = parse_alias(&to_string(&pbuf.components().next().unwrap())).unwrap();
                    
                    let sp_depend_path = shortpaths.get(&sp_depend_name).unwrap();
                    debug!("sp_depend_name = {}", &sp_depend_name);
                    let depend_path = sp_depend_path.path.to_str().unwrap().to_string();
                    debug!("sp_depend_path = {}", &depend_path);

                    let expanded = expand_path(entry.to_str().unwrap(), &sp_depend_name, &depend_path);
                    trace!("Expanding layer: {} -> {}", entry.display(), &expanded);

                    output = expanded.clone();

                    trace!("Expanding all layers...");
                    trace!("");
                    let result = f(sp_depend_name, expanded, PathBuf::from(output), true, shortpaths);
                    trace!("B2: Received result: {}", &result);
                    output = result.clone();
                    return result;

                } else {
                    trace!("Inside Termination Case");
                    trace!("Branch 3");
                    trace!("Alias Path: {}", &alias_path);
                    trace!("");

                    let pbuf = PathBuf::from(alias_path);
                    let sp_depend_name = parse_alias(&to_string(&pbuf.components().next().unwrap())).unwrap();
                    debug!("sp_depend_name = {}", &sp_depend_name);

                    let sp_depend_path = shortpaths.get(&sp_depend_name).unwrap();
                    let depend_path = sp_depend_path.path.to_str().unwrap().to_string();
                    debug!("sp_depend_path = {}", &depend_path);

                    let expanded = expand_path(entry.to_str().unwrap(), &sp_depend_name, &depend_path);
                    trace!("Expanding layer: {} -> {}", entry.display(), &expanded);

                    trace!("All Layers Expanded");
                    return expanded;
                }
            }
        }
        debug!("Output: {}", output);
        output
    }

    let str_path = f(String::new(), String::new(), sp.path.to_owned(), false, shortpaths);
    PathBuf::from(str_path)
}

// Commands
pub fn add_shortpath(shortpaths: &mut SP, name: String, path: PathBuf) {
    let shortpath = Shortpath::new(path, None);
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
        let shortpath = Shortpath::new(PathBuf::from(new_path), None);
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
