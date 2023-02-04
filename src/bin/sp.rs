use shortpaths::export::get_exporter;
use std::{
    path::{PathBuf, Component},
    env::var,
    cmp::Ordering,
};

use indexmap::IndexMap;

type SP = IndexMap<String, Shortpath>;
type SPT = ShortpathType;
type DEPS = Vec<SPT>; 

/// The type of shortpath it is
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortpathType {
    Path(String, PathBuf),      // Shortpath Name   : Shortpath Path
    AliasPath(String, PathBuf), // Shortpath Name   : Shortpath Path
    EnvPath(String, PathBuf),   // Env Var Name     : Shortpath Path
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortpath {
    path: SPT,
    full_path: Option<PathBuf>,
    deps: Option<DEPS>,
}

impl Ord for Shortpath {
    fn cmp(&self, other: &Self) -> Ordering {
        // The order that shortpaths are determined are:
        // 1. Length of dependencies
        // 2. Lexicographical order
        // NOTE:
        // Ideally, we should order shortpaths around their closest matching paths
        // We can add this later with the strsim crate potentially
        let (len_deps_1, len_deps_2) = (self.deps.as_ref().unwrap().len(), other.deps.as_ref().unwrap().len());
        let (len_name_1, len_name_2) = (get_shortpath_name(&self.path), get_shortpath_name(&other.path));

        len_deps_1.cmp(&len_deps_2)
            .then(len_name_1.cmp(&len_name_2))
    }
}

impl PartialOrd for Shortpath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct ShortpathsBuilder {
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
        Some(spt) => {spt}
        None => { SPT::Path(name.clone(), path) }
    };
    let shortpath = Shortpath::new(spt, None, None);
    shortpaths.insert(name, shortpath);
}

pub fn remove_shortpath(shortpaths: &mut SP, current_name: &str) {
    shortpaths.remove(current_name);
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

//  TODO: Implement autoindex

/* /** Serialize shortpaths to other formats for use in other applications */
*/
//pub fn export(shortpaths: &mut SP, export_type: &str, output_file: Option<&String>) -> String {
    //let mut exp = get_exporter(export_type.into());
    //exp.set_shortpaths_imap(shortpaths);
    
    //let dest = match output_file {
        //Some(path)  => Path::new(path).to_path_buf(),
        //None        => PathBuf::from(exp.get_completions_path())
    //};

    //fs::create_dir_all(dest.parent().expect("Could not get parent directory"))
        //.expect("Could not create shell completions directory");

    //// Serialize
    //let output = exp.gen_completions();
    //fs::write(&dest, &output).expect("Unable to write to disk");
    //dest.to_str().unwrap().to_owned()
//}



//pub fn add(shortpaths: SP, alias_path: &Path) -> Overwritten<String, PathBuf> {
    //self.shortpaths.insert(alias_name.into(), alias_path.into())
//}


/*
 * Operations on shortpaths
 * - Add, Remove, Update, Check, Export
 * Add:
 *  - Should not reorder the entire file
 *  - Validate the path
 *  - Append to file
 * Remove:
 *  - Should not reorder the entire file
 *  - Validate the path
 *  - Remove the entry from the file
 * Update:
 *  - Validates the existence of a name (l)or a path
 *  - Modify just the name or the path in the file
 */

fn main() {
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

     // Test serialization
     println!("Sorted list of shortpaths");
     let sorted = sort_shortpaths(sp_im);
     sorted.iter().for_each(|p| println!("{:?}", p));
}
