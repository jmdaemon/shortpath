use crate::app::{ExportType, Mode, ResolveType};
use crate::builder::{Shortpaths, ShortpathsAlignExt};
use crate::env::{EnvPathOperationsExt, EnvVars};
use crate::export::get_exporter;
use crate::helpers::{
    to_str_slice,
    search_for,
    matching_file_names,
    in_parent_dir,
    auto_resolve,
    manual_resolve, ScopeResults,
    prompt_until_valid,
};

use std::{
    path::{Path, PathBuf, Component},
    cmp::Ordering,
    process::exit,
};

#[allow(unused_imports)]
use itertools::Itertools;
use indexmap::IndexMap;
use log::{trace, debug, info};
use serde::{Serialize, Serializer, Deserialize, Deserializer};

// Data Types
pub type SP = IndexMap<String, Shortpath>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortpathVariant {
    Independent,
    Alias,
    Environment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortpath {
    pub path: PathBuf,
    pub full_path: Option<PathBuf>,
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

// Trait Extensions
// We need these for remove_hook, move_hook to check shortpath configs
pub trait FindKeyIndexMapExt<'a, K, V> {
    /// Get keys from value of IndexMap
    fn find_keys_for_value(&'a self, value: V) -> Vec<&'a K>;

    /// Get key from value of IndexMap
    fn find_key_for_value(&'a self, value: V) -> Option<&'a K>;

/// Get key from value of IndexMap
    fn find_key_for_full_path(&'a self, value: V) -> Option<&'a K>;
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

    fn find_key_for_full_path(&'a self, value: V) -> Option<&'a String> {
        let v = value.into();
        self.iter().find_map(|(key, val)| if val.full_path.as_ref()?.to_str().unwrap() == v { Some(key) } else { None })
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

pub fn substitute_env_path(src: &str, env_name: &str, env_path: &str) -> String {
    let this = format!("${{env:{}}}", env_name);
    let with = env_path;
    src.replace(&this, with)
}

pub fn substitute_env_alias_name(src: &str, env_name: &str) -> String {
    let this = format!("${{env:{}}}", env_name);
    let with = format!("${}", env_name);
    src.replace(&this, &with)
}

pub fn sort_shortpaths(shortpaths: SP) -> SP {
    shortpaths.sorted_by(|_, v1, _, v2| {
        v1.cmp(v2)
    }).collect()
}

// Input Parsing

/** Return the type of a shortpath entry */
pub fn get_shortpath_type(comp: &[char]) -> Option<ShortpathVariant> {
    let _env_prefix = to_str_slice("${env:");
    let is_not_empty = |path: &[char]| { !path.is_empty() };

    match (is_not_empty(comp), comp) {
        (true, [ _env_prefix, .., '}']) => Some(ShortpathVariant::Environment),
        (true, ['$', ..])               => Some(ShortpathVariant::Alias),
        (true, [ .. ])                  => Some(ShortpathVariant::Independent),
        (false, _)                      => None
    }
}

/// Parse a component of a path, and determine any shortpath variants
pub fn parse_alias(comp: String) -> Option<String> {
    if comp.starts_with('$') {
        Some(comp.split_at(1).1.to_owned())
    } else {
        None
    }
}

/// Parse a component of an environment path, and returns it
pub fn parse_env_alias(comp: String) -> Option<String> {
    if comp.starts_with("${env:") {
        let mut envname = comp.strip_prefix("${env:").unwrap().to_owned();
        envname.pop();
        Some(envname)
    } else {
        None
    }
}

/// Replaces ${env:PATH} -> $PATHs
pub fn substitute_env_paths(shorpaths: SP) -> SP {
    shorpaths.into_iter().map(|(name, mut sp)| {
        let mut path = sp.path.to_str().unwrap().to_owned();
        sp.path.components().for_each(|comp| {
            let compstr = comp.as_os_str().to_str().unwrap().to_owned();
            let alias = parse_env_alias(compstr);
            if let Some(env_name) = alias {
                path = substitute_env_alias_name(&path, &env_name);
            }
        });
        sp.path = PathBuf::from(path);
        (name, sp)
    }).collect()
}

pub fn to_string(comp: &Component) -> String {
    comp.as_os_str().to_str().unwrap().to_string()
}

pub fn str_join_path(s1: &str, s2: &str) -> PathBuf {
    let (p1,p2) = (PathBuf::from(s1), PathBuf::from(s2));
    p1.join(p2)
}

pub fn get_sp_deps(alias_name: String, shortpaths: &SP) -> (String, String) {
    let sp_depend_name = parse_alias(alias_name).unwrap();
    debug!("sp_depend_name = {}", &sp_depend_name);

    let sp_depend_path = shortpaths.get(&sp_depend_name).unwrap();
    let depend_path = sp_depend_path.path.to_str().unwrap().to_string();
    debug!("depend_path = {}", &depend_path);
    (sp_depend_name, depend_path)
}

pub fn fold_shortpath(mut path: PathBuf, shortpaths: &SP) -> PathBuf {
    info!("fold_shortpath()");
    let mut output = path.to_str().unwrap().to_owned();
    while let Some(parent) = path.parent() {
        if let Some(key) = shortpaths.find_key_for_full_path(parent.to_str().unwrap()) {
            let this = parent.to_str().unwrap();
            let with = format!("${}", key);
            output = output.replace(this, &with);
        }
        path = parent.to_owned();
    }
    PathBuf::from(output)
}

/**
  * Expand shortpath variants at runtime
  * 
  * The shortpath dependencies must be populated before this is run. When they are
  * populated, both their name and path values are stored in the enum variant which
  * is accessed here without the hashmap.
  */
pub fn expand_shortpath(sp: &Shortpath, shortpaths: &SP) -> PathBuf {
    info!("expand_shortpath()");

    // Helper functions for use in expand_shortpath_inner
    fn get_sp_alias_name_base(comp: Component) -> String { to_string(&comp) }

    /// NOTE: This only works for the first component of a string
    /// and it's highly likely that in the future, this will have to change
    /// to support more complex shortpaths (doubly nested paths like $a/$b)
    fn get_sp_alias_name_recurse(alias_path: String) -> String {
        let pbuf = PathBuf::from(alias_path);
        to_string(&pbuf.components().next().unwrap())
    }

    fn get_expanded_path(entry: PathBuf, sp_depend_name: &str, depend_path: &str) -> String {
        let expanded = expand_path(entry.to_str().unwrap(), sp_depend_name, depend_path);
        debug!("\tExpand: {} -> {}", entry.display(), &expanded);
        expanded
    }

    fn expand_layer(alias_name: String, depend_path: String, entry: PathBuf, shortpaths: &SP) -> String {
        let result = expand_shortpath_inner(alias_name, depend_path, entry, true, shortpaths);
        debug!("\tResult: {}\n", result);
        result
    }

    pub fn expand_shortpath_inner(alias_name: String, alias_path: String, entry: PathBuf, has_started: bool, shortpaths: &SP) -> String {
        info!("Inputs");
        debug!("\talias_path  = {}", &alias_path);
        debug!("\tentry       = {}", &entry.display());
        debug!("\talias_name  = {}\n", &alias_name);

        let peek_clone = entry.clone();
        let mut peekable = peek_clone.components().peekable();
        if has_started && (alias_name.is_empty() || peekable.peek().is_none()) {
            return alias_path;
        }
        // Else Assume we can obtain a component
        let comp = entry.components().next().unwrap();
        let comp_slice = to_str_slice(comp.as_os_str().to_str().unwrap());
        let shortpath_type = get_shortpath_type(&comp_slice);

        // If we have nothing, exit early
        match &shortpath_type {
            Some(var) => {
                match var {
                    ShortpathVariant::Environment | ShortpathVariant::Independent => return alias_path,
                    _ => {}
                }
            }
            None => return alias_path
        }
        // Else Assume we can obtain a variant
        let shortpath_variant = shortpath_type.unwrap();

        let mut expanded = String::new();
        match shortpath_variant {
            ShortpathVariant::Environment => {
                trace!("Environment Variable Detected");
            }
            ShortpathVariant::Alias => {
                if !has_started {
                    info!("Branch 1: Beginning recursive expansion");
                    let (sp_depend_name, depend_path)  = get_sp_deps(get_sp_alias_name_base(comp), shortpaths);
                    expanded = get_expanded_path(entry, &sp_depend_name, &depend_path);

                    return expand_layer(format!("${}", &sp_depend_name), expanded.clone(), PathBuf::from(expanded), shortpaths);
                } else if let Some(parsed) = parse_alias(alias_name) {
                    trace!("Branch 2: In recursive expansion");
                    trace!("Parsed alias_name: {}", &parsed);

                    let (sp_depend_name, depend_path) = get_sp_deps(get_sp_alias_name_recurse(alias_path), shortpaths);
                    expanded = get_expanded_path(entry, &sp_depend_name, &depend_path);
                    return expand_layer(sp_depend_name, expanded.clone(), PathBuf::from(expanded), shortpaths);
                } else {
                    trace!("Branch 3: Inside Termination Case");
                    trace!("Alias Path: {}", &alias_path);

                    let (sp_depend_name, depend_path) = get_sp_deps(get_sp_alias_name_recurse(alias_path), shortpaths);
                    expanded = get_expanded_path(entry, &sp_depend_name, &depend_path);
                    trace!("All Layers Expanded");
                    return expanded;
                }
            }
            _ => {}
        }
        debug!("Expanded: {}", expanded);
        expanded
    }
    let entry = sp.path.to_str().unwrap().to_string();
    let str_path = expand_shortpath_inner(String::new(), entry, sp.path.to_owned(), false, shortpaths);

    PathBuf::from(str_path)
}

// Commands
pub fn add_shortpath(shortpaths: &mut SP, name: String, path: PathBuf) {
    let shortpath = Shortpath::new(path.clone(), Some(path));
    shortpaths.insert(name, shortpath);
}

pub fn remove_shortpath(shortpaths: &mut SP, names: &[String], yes: bool) -> Vec<Option<Shortpath>> {
    let mut removed = vec![];
    let is_valid_input = |input: String| {
        matches!(input.as_str().trim_end(), "yes" |  "no")
    };

    for name in names.iter() {
        if !yes {
            let sp = shortpaths.get(name).unwrap();
            let path = sp.path.display();
            let message = format!("Remove {} : {}? [yes/no]: ", name, path);

            let input = prompt_until_valid(&message, is_valid_input);
            match input.as_str() {
                "yes" => {
                    let sp = shortpaths.remove(name);
                    removed.push(sp);
                },
                "no" => continue,
                _ => continue,
            };
        } else {
            let sp = shortpaths.remove(name);
            removed.push(sp);
        }
    }
    removed
}

pub fn find_unreachable(shortpaths: &SP) -> IndexMap<String, Shortpath> {
    let unreachable: IndexMap<String, Shortpath> = shortpaths.into_iter()
        .filter(|(_, sp)| {
            let full_path = &sp.full_path;
            full_path.is_none() || !full_path.as_ref().unwrap().exists()
        }).into_iter().map(|(name, sp)| (name.to_owned(), sp.to_owned())).collect();
    unreachable
}

/// List any broken or unreachable paths
pub fn check_shortpaths(shortpaths: &mut SP) {
    let unreachable = find_unreachable(shortpaths);
    unreachable.iter().for_each(|(alias_name, alias_path)|
        println!("{} shortpath is unreachable: {}", alias_name, alias_path.path.display()));
    println!("Check Complete");
}

/// Displays a shortpath
/// NOTE: In the future this function could potentially accept another
/// parameter 'pretty_print'
pub fn display_shortpath(name: &str, path: &Path) {
    println!("{}: {}", name, path.display());
}

/// List saved shortpaths
pub fn show_shortpaths(shortpaths: &Shortpaths, names: Option<Vec<String>>) {
    match names {
        Some(names) => {
            // Print the names of all the desired shortpaths
            names.iter().for_each(|name| {
                if let Some(shortpath) = shortpaths.shortpaths.get(name) {
                    println!("{} : {}", name, shortpath.path.display());
                } else {
                    println!("Could not find {}", name);
                    // TODO: Print clap usage
                }
            });
        }
        None => {
            // Dump the pretty printed config
            let config = shortpaths.tab_align_paths();
            print!("{}", config);
        }
    }
}

pub fn show_unreachable(unreachable: &SP) {
    debug!("Unreachable Shortpaths: ");
    unreachable.iter().for_each(|(k, sp)| {
        if let Some(full_path) = &sp.full_path {
            debug!("\tname      : {}", k);
            debug!("\tpath      : {}", sp.path.display());
            debug!("\tfull_path : {}", full_path.display());
        } else {
            debug!("\tname      : {}", k);
            debug!("\tpath      : {}", sp.path.display());
        }
    });
}

pub fn show_search_results(results: &IndexMap<String, ScopeResults>) {
    // Show output
    // TODO: Make this output less noisy when they are no paths found
    debug!("Showing Results");
    results.iter().for_each(|(name, nested_entries)| {
        //let mut search_results_peek = nested_entries.iter().peekable();
        //let next_peek = search_results_peek.peek().unwrap();
        //let entry_peek = &next_peek.1;
        //if !entry_peek.is_empty() {
            debug!("Unreachable Path: {}", name);
            nested_entries.iter().for_each(|(dirname, search_results)| {
                debug!("Directory {}", dirname.display());
                debug!("Files Found:");
                search_results.iter().for_each(|file| {
                    debug!("\t{}", file.path().display());
                });
            });
        //}
    });
}

/** Fix unreachable or broken paths
  * 
  * `resolve` makes use of search and scope functions to provide more
  *     fine grained searching functionality.
  * 
  * The currently supported execution modes are:
  * Search Functions: 
  *     - matching_file_names 
  *     - similar_file_names (TODO NOT IMPLEMENTED).
  * Scope Functions: 
  *     - in_parent_dir 
  *     - nearest_neighbours (TODO NOT IMPLEMENTED).
  * Resolve Modes:
  *     - Automode  : Selects the first/best possible candidate, 
  *     - Manual    : Defer resolve choice to the user.
  */
pub fn resolve(shortpaths: &mut SP, resolve_type: ResolveType, mode: Mode, dry_run: bool) {
    info!("resolve()");

    let unreachable = find_unreachable(shortpaths);
    if unreachable.is_empty() {
        debug!("None found");
        println!("No unreachable paths found");
        exit(0);
    }

    // TODO: Create Display for Unreachable type
    show_unreachable(&unreachable);
    
    // Select search & scope functions
    let search_fn = match resolve_type {
        ResolveType::Matching => matching_file_names,
    };

    let scope_fn = in_parent_dir;

    debug!("Parameters");
    debug!("\tresolve_type: {:?}", resolve_type);
    debug!("\tmode        : {:?}", mode);
    debug!("\tdry_run     : {}", dry_run);

    debug!("Attempting to search for files...");
    let results: IndexMap<String, ScopeResults> = search_for(search_fn, scope_fn, &unreachable);

    // Exit early if no matches found
    if results.is_empty() {
        println!("No matches found.");
        exit(0);
    }
    show_search_results(&results);

    // Store the updates to make
    let mut updates: Vec<(String, PathBuf, PathBuf)> = Vec::new();

    for (name, sp) in unreachable.iter() {
        let previous = sp.to_owned().full_path.unwrap();
        for (_, nested_entries) in &results {
            let choice = match mode {
                Mode::Automatic => auto_resolve(name.to_owned(), nested_entries.to_owned()),
                Mode::Manual => manual_resolve(name.to_owned(), &previous.clone(), nested_entries.to_owned()),
            };

            if let Some(updated) = choice {
                updates.push((name.to_owned(), previous.clone(), updated.1));
            }
        }
    }

    for (name, _, updated) in updates.into_iter() {
        debug!("Name    : {name}");
        debug!("Updated : {}", updated.display());
        update_shortpath(shortpaths, &name, None, Some(updated))
    }
}

/** Serialize shortpaths to other formats for use in other applications */
pub fn export_shortpaths(shortpaths: &SP, export_type: ExportType, output_file: Option<PathBuf>) -> PathBuf {
    // Sets environment variables
    let mut evars = EnvVars::new();
    let vars = evars.vars.non_null().unique(shortpaths).strict();
    evars.vars = vars;

    let exp = get_exporter(export_type);

    let dest = exp.prepare_directory(output_file);
    exp.write_completions(&dest, shortpaths.to_owned())
}

pub fn update_shortpath_name(current_name: &str, new_name: String, shortpaths: &mut SP) {
    let path = shortpaths.remove(current_name).unwrap();
    shortpaths.insert(new_name, path);
}

pub fn update_shortpath_path(current_name: &str, new_path: PathBuf, full_path: Option<PathBuf>, shortpaths: &mut SP) {
    let shortpath = Shortpath::new(new_path, full_path);
    shortpaths.insert(current_name.to_owned(), shortpath);
}

/** Update a single shortpath's alias name or path
  * Changes the name or path if given and are unique */
pub fn update_shortpath(shortpaths: &mut SP, current_name: &str, name: Option<String>, path: Option<PathBuf>) {
    let entry_exists = || { shortpaths.get(current_name).is_some() }; 
    match (entry_exists(), name, path) {
        (true, Some(new_name), _) => { update_shortpath_name(current_name, new_name, shortpaths); }
        (true, _, Some(new_path)) => { update_shortpath_path(current_name, new_path.clone(), Some(new_path), shortpaths); }
        (_, _, _)              => { println!("Nothing to do");}
    }
}
