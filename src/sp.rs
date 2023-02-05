use crate::export::get_exporter;
use std::{
    path::{Path, PathBuf, Component},
    env::var,
    cmp::Ordering,
    fs::{create_dir_all, write},
};

use indexmap::IndexMap;
use itertools::Itertools;
use log::{debug, trace};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use walkdir::{DirEntry, WalkDir};

pub type SP = IndexMap<String, Shortpath>;
pub type SPT = ShortpathType;
pub type DEPS = Vec<SPT>; 

/// The type of shortpath it is
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortpathType {
    Path(String, PathBuf),      // Shortpath Name   : Shortpath Path
    AliasPath(String, PathBuf), // Shortpath Name   : Shortpath Path
    EnvPath(String, PathBuf),   // Env Var Name     : Shortpath Path
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortpath {
    path: SPT,
    #[serde(skip)]
    full_path: Option<PathBuf>,
    #[serde(skip)]
    deps: Option<DEPS>,
}
//impl Serialize for ShortpathType {
    //fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    //where
        //S: Serializer,
    //{
        //serializer.serialize_str(&get_shortpath_name(&self))


// Implementations
impl Serialize for ShortpathType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(get_shortpath_path(&self).to_str().unwrap())
    }
}

impl<'de> Deserialize<'de> for ShortpathType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        // do better hex decoding than this
        //u64::from_str_radix(&s[2..], 16)
            //.map(Account)
            //.map_err(D::Error::custom)
        let path = PathBuf::from(s);
        // This may cause problems for us later since we may not have the name field available to us here
        Ok(ShortpathType::Path(String::new(), path))
    }
}
        //serializer.serialize_str(&get_shortpath_name(&self))
        //serializer.serialize_str(get_shortpath_path(&self).to_str().unwrap());

        ////let mut map = serializer.serialize_map(Some(self.x.len()))?;
        ////for (k, v) in &self.x {
            ////map.serialize_entry(&k.to_string(), &v)?;
        ////}
        ////map.end()
    //}
//}

//impl Serialize for Shortpath {
    //fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    //where
        //S: Serializer,
    //{
        //serializer.serialize_str(self.)

        ////let mut map = serializer.serialize_map(Some(self.x.len()))?;
        ////for (k, v) in &self.x {
            ////map.serialize_entry(&k.to_string(), &v)?;
        ////}
        ////map.end()
    //}
//}


impl Ord for Shortpath {
    fn cmp(&self, other: &Self) -> Ordering {
        // The order that shortpaths are determined are:
        // 1. Length of dependencies
        // 2. Path lexicographical order
        // 3. Name lexicographical order
        // NOTE:
        // Ideally, we should order shortpaths around their closest matching paths
        // We can add this later with the strsim crate potentially
        let (mut len_deps_1, mut len_deps_2) = (0, 0);
        let (len_path_1, len_path_2) = (get_shortpath_path(&self.path), get_shortpath_path(&other.path));
        let (len_name_1, len_name_2) = (get_shortpath_name(&self.path), get_shortpath_name(&other.path));

        if self.deps.is_some() {
            len_deps_1 = self.deps.as_ref().unwrap().len();
        }

        if other.deps.is_some() {
            len_deps_2 = other.deps.as_ref().unwrap().len();
        }

        //if self.deps.is_some() && other.deps.is_some() {
            //(len_deps_1, len_deps_2) = (self.deps.as_ref().unwrap().len(), other.deps.as_ref().unwrap().len());
        //}

        len_deps_1.cmp(&len_deps_2)
            .then(len_path_1.cmp(&len_path_2))
            .then(len_name_1.cmp(&len_name_2))
            .reverse()
    }
}

impl PartialOrd for Shortpath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ShortpathsBuilder {
    paths: Option<SP>,
}

// Implementations

impl ShortpathType {
    pub fn new_path(name: impl Into<String>, path: impl Into<PathBuf>) -> SPT {
        SPT::Path(name.into(), path.into())
    }
    pub fn new_alias_path(name: impl Into<String>, alias_path: impl Into<PathBuf>) -> SPT {
        SPT::AliasPath(name.into(), alias_path.into())
    }
    pub fn new_env_path(name: impl Into<String>) -> SPT {
        // Store environment variable values during creation
        let name = name.into();
        let mut alias_path = String::new();
        match var(&name) { // Get from environment
            Ok(val) => alias_path = val,
            Err(e) => eprintln!("Error in expanding environment variable ${}: ${}", name, e)
        };
        SPT::EnvPath(name, alias_path.into())
    }
}

impl Shortpath {
    pub fn new(path: SPT, full_path: Option<PathBuf>, deps: Option<DEPS>) -> Shortpath {
        Shortpath { path: path.into(), full_path, deps }
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
        let builder = ShortpathsBuilder { paths: Some(im) };
        builder
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

            shortpaths.iter_mut().for_each(|(_, sp)| {
                sp_pop_deps(sp);
                sp_pop_full_path(sp);
            });

            return Some(shortpaths.to_owned());
        }
        None
    }
}

// Trait Extensions
// NOTE: We may not need these in the future, so they could be potentially removed
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
            .filter_map(|(key, val)| if val.path().to_str().unwrap().to_owned() == v { Some(key) } else { None })
            .collect()
    }

    fn find_key_for_value(&'a self, value: V) -> Option<&'a String> {
        let v = value.into();
        self.iter().find_map(|(key, val)| if val.path().to_str().unwrap().to_owned() == v { Some(key) } else { None })
    }
}

// Pure Functions

// General Purpose
/// Convert strings into a vector of characters
pub fn to_str_slice(s: impl Into<String>) -> Vec<char> {
    s.into().chars().collect()
}

pub fn find_longest_keyname<T>(map: IndexMap<String, T>) -> String {
    map.into_iter()
       .max_by(|(k1,_), (k2, _)| k1.len().cmp(&k2.len()))
       .unwrap().0.to_owned()
}

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

// Shortpaths Specific

pub fn fmt_expand(src: &str, key_name: &str, key_path: &str) -> String {
    let this = format!("${}", key_name);
    let with = key_path;
    src.replace(&this, &with)
}

pub fn fmt_fold(src: &str, key_name: &str, key_path: &str) -> String {
    let this = key_path;
    let with = format!("${}", key_name);
    src.replace(&this, &with)
}

pub fn get_shortpath_name(sp: &SPT) -> String {
    match sp {
        SPT::Path(name, _) | SPT::AliasPath(name, _) | SPT::EnvPath(name, _) => name.to_owned(),
    }
}

pub fn get_shortpath_path(sp: &SPT) -> PathBuf {
    match sp {
        SPT::Path(_, path) | SPT::AliasPath(_, path) | SPT::EnvPath(_, path) => path.to_owned()
    }
}

// Input Parsing
/** Parse a Shortpath entry, and returns any dependencies */
pub fn parse_alias(path: &[char]) -> Option<SPT> {
    match path {
        ['$', alias_name @ ..] => {
            let (an, ap) = (alias_name.iter().collect(), PathBuf::from(path.iter().collect::<String>()));
            Some(SPT::AliasPath(an, ap))
        }
        [ '{', '$', 'e', 'n', 'v', ':', alias_name @ .., '}'] => {
            let (an, ap) = (alias_name.iter().collect(), PathBuf::from(path.iter().collect::<String>()));
            Some(SPT::EnvPath(an, ap))
        }
        _ => { None }
    }
}

pub fn get_shortpath_type(name: impl Into<String>, path: &PathBuf) -> SPT {
    let spt = match &path.to_str().unwrap().to_owned().to_lowercase() {
        keyword if keyword.contains("$")        => SPT::AliasPath(name.into(), path.to_owned()),
        keyword if keyword.contains("${env:")   => SPT::EnvPath(name.into(), path.to_owned()),
        _                                       => SPT::Path(name.into(), path.to_owned())
    };
    spt
}

/// Find the dependencies for a given shortpath
pub fn find_deps(entry: &PathBuf) -> Option<DEPS> {
    let deps: DEPS = entry.components().into_iter().filter_map(|path_component| {
        if let Component::Normal(osstr_path) = path_component {
            return parse_alias(&to_str_slice(osstr_path.to_string_lossy()));
        }
        return None
    }).collect();
    Some(deps)
}

/**
  * Expand shortpath variants at runtime
  * 
  * The shortpath dependencies must be populated before this is run. When they are
  * populated, both their name and path values are stored in the enum variant which
  * is accessed here without the hashmap.
  */
pub fn expand_shortpath(sp: &Shortpath) -> String {
    let entry = sp.path().to_str().unwrap().to_owned();
    let mut output = entry.clone();
    match &sp.deps {
        Some(deps) => // Expand entry into full_path
            deps.iter().for_each(|dep| {
                // TODO: Wrap in a while loop later to parse additional paths
                let (dep_name, dep_path) = (get_shortpath_name(dep), get_shortpath_path(dep));
                output = fmt_expand(&output, &dep_name, dep_path.to_str().unwrap());
            }),
        None => output = entry
    };
    output
}

// Impure Shortpath Functions
/// Populate shortpath dependencies
pub fn sp_pop_deps(sp: &mut Shortpath) {
    if sp.deps.is_some() { return; }
    sp.deps = find_deps(&sp.path());
}

/// Expand and populate shortpath's full_path field
pub fn sp_pop_full_path(sp: &mut Shortpath) {
    let output = expand_shortpath(sp);
    sp.full_path = Some(PathBuf::from(output));
}

pub fn sort_shortpaths(shortpaths: SP) -> SP {
    shortpaths.sorted_by(|_, v1, _, v2| {
        v1.cmp(v2)
    }).collect()
}

// Commands
//pub fn add(shortpaths: &mut SP, name: impl Into<String>, path: impl Into<PathBuf>) {
pub fn add_shortpath(shortpaths: &mut SP, name: String, path: PathBuf) {
    let spt = match parse_alias(&to_str_slice(path.to_str().unwrap())) {
        Some(spt) => spt,
        None => SPT::Path(name.clone(), path)
    };
    let shortpath = Shortpath::new(spt, None, None);
    shortpaths.insert(name, shortpath);
}

pub fn remove_shortpath(shortpaths: &mut SP, current_name: &str) -> Option<Shortpath> {
    return shortpaths.remove(current_name)
}

pub fn find_unreachable(shortpaths: &SP) -> IndexMap<&String, &Shortpath> {
    let unreachable: IndexMap<&String, &Shortpath> = shortpaths.iter()
        .filter(|(_, path)| { if !path.path().exists() || path.path().to_str().is_none() { true } else { false }
        }).collect();
    unreachable
}

/// List any broken or unreachable paths
pub fn check_shortpaths(shortpaths: &mut SP) {
    let unreachable = find_unreachable(shortpaths);
    unreachable.iter().for_each(|(alias_name, alias_path)|
        println!("{} shortpath is unreachable: {}", alias_name, alias_path.path().display()));
    println!("Check Complete");
}

// File search functions

/// Attempt to find the a file in a dir
pub fn find_by_matching_path(file_name: &str, dir: WalkDir) -> Vec<DirEntry> {
    let files: Vec<DirEntry> = dir.into_iter()
        .filter_map(Result::ok)
        .filter(|file| file.file_name() == file_name)
        .collect();
    files.to_owned()
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

        if files.len() > 0 {
            return Some(files);
        }
        next = dir.parent(); // Continue searching
    }
    None
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
pub fn export_shortpaths(shortpaths: &SP, export_type: &str, output_file: Option<&String>) -> String {
    let mut exp = get_exporter(export_type.into());
    exp.set_shortpaths(shortpaths);
    
    let dest = match output_file {
        Some(path)  => Path::new(path).to_path_buf(),
        None        => PathBuf::from(exp.get_completions_path())
    };

    create_dir_all(dest.parent().expect("Could not get parent directory"))
        .expect("Could not create shell completions directory");

    // Serialize
    let output = exp.gen_completions();
    write(&dest, &output).expect("Unable to write to disk");
    dest.to_str().unwrap().to_owned()
}

/** Update a single shortpath's alias name or path
  * Changes the name or path if given and are unique */
pub fn update_shortpath(shortpaths: &mut SP, current_name: &str, name: Option<&String>, path: Option<&String>) {
    if let Some(new_path) = path {
        let path = PathBuf::from(new_path);
        
        let spt = match parse_alias(&to_str_slice(path.to_str().unwrap())) {
            Some(spt) => spt,
            None => SPT::Path(current_name.to_owned(), path)
        };
        let shortpath = Shortpath::new(spt, None, None);
        shortpaths.insert(current_name.to_owned(), shortpath);
    } else if let Some(new_name) = name {
        let path = shortpaths.remove(current_name).unwrap();
        shortpaths.insert(new_name.to_owned(), path);
    } 
}

#[test]
fn test_shortpaths() {
    use crate::sp::{
        SPT,
        Shortpath,
        ShortpathsBuilder,
        FindKeyIndexMapExt,
        sort_shortpaths,
        export_shortpaths,
    };

    use std::path::PathBuf;

    // TODO Create more ergonomic api for this later
    // Wrap it together with the builder construct to reduce the noise
    let sp_paths = vec![
        Shortpath::new(SPT::new_path("d", PathBuf::from("$a/dddd")), None, None),
        Shortpath::new(SPT::new_path("c", PathBuf::from("$b/cccc")), None, None),
        Shortpath::new(SPT::new_path("b", PathBuf::from("$a/bbbb")), None, None),
        Shortpath::new(SPT::new_path("a", PathBuf::from("aaaa")), None, None),
    ];
    println!("{:?}", sp_paths);

    let mut sp_builder = ShortpathsBuilder::new(sp_paths);

    let sp_im = sp_builder.build().unwrap();
    sp_im.iter().for_each(|p| println!("{:?}", p));

    // Test find_key
    let key = sp_im.find_key_for_value("$a/bbbb");
    println!("{:?}", key);

    let key = sp_im.find_key_for_value("$a/bbbb".to_string());
    println!("{:?}", key);

    // Test sort_shortpaths
    println!("Sorted list of shortpaths");
    let sorted = sort_shortpaths(sp_im);
    sorted.iter().for_each(|p| println!("{:?}", p));

    // Test serialization
    let export_type = "bash";
    let output_file = None;
    export_shortpaths(&sorted, export_type, output_file);
}
