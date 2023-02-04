use std::path::{PathBuf, Component};
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
    path: ShortpathType,
    full_path: Option<PathBuf>,
    deps: Option<DEPS>,
}

struct ShortpathsBuilder {
    paths: Option<SP>,
}

// Implementations

impl ShortpathType {
    pub fn new_path(name: impl Into<String>, path: impl Into<PathBuf>) -> ShortpathType {
        ShortpathType::Path(name.into(), path.into())
    }
    pub fn new_alias_path(name: impl Into<String>, alias_path: impl Into<PathBuf>) -> ShortpathType {
        ShortpathType::AliasPath(name.into(), alias_path.into())
    }
    pub fn new_env_path(name: impl Into<String>) -> ShortpathType {
        // Store environment variable values during creation
        let name = name.into();
        let mut alias_path = String::new();
        match std::env::var(&name) { // Get from environment
            Ok(val) => alias_path = val,
            Err(e) => eprintln!("Error in expanding environment variable ${}: ${}", name, e)
        };
        ShortpathType::EnvPath(name, alias_path.into())
    }
}

impl Shortpath {
    pub fn new(path: ShortpathType, full_path: Option<PathBuf>, deps: Option<DEPS>) -> Shortpath {
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

// Shortpaths Specific
/// Find the longest possible keyname in the hashmap
pub fn find_longest_keyname<T>(map: IndexMap<String, T>) -> String {
    map.into_iter()
       .max_by(|(k1,_), (k2, _)| k1.len().cmp(&k2.len()))
       .unwrap().0.to_owned()
}

/// Expand the shortpath
pub fn fmt_expand(src: &str, key_name: &str, key_path: &str) -> String {
    let this = format!("${}", key_name);
    let with = key_path;
    src.replace(&this, &with)
}

/// Fold the shortpath
pub fn fmt_fold(src: &str, key_name: &str, key_path: &str) -> String {
    let this = key_path;
    let with = format!("${}", key_name);
    src.replace(&this, &with)
}

// Input Parsing
/** Determines shortpath dependencies from the shortpath entry */
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

// Owned versions of Shortpath name(), path()

pub fn get_shortpath_name(sp: &SPT) -> String {
    match sp {
        SPT::Path(name, _) | SPT::AliasPath(name, _) | SPT::EnvPath(name, _) => name.to_owned(),
    }
}

pub fn get_shortpath_path(sp: &SPT) -> PathBuf {
    match &sp {
        SPT::Path(_, path) | SPT::AliasPath(_, path) | SPT::EnvPath(_, path) => path.to_owned()
    }
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
        None => output = entry // Use the entry as the full_path
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

// How do we add 

fn main() {
    // TODO Create more ergonomic api for this later
    // Wrap it together with the builder construct to reduce the noise
     let sp_paths = vec![
         Shortpath::new(SPT::new_path("a", PathBuf::from("aaaa")), None, None),
         Shortpath::new(SPT::new_path("b", PathBuf::from("$a/bbbb")), None, None),
         Shortpath::new(SPT::new_path("c", PathBuf::from("$b/cccc")), None, None),
         Shortpath::new(SPT::new_path("d", PathBuf::from("$a/dddd")), None, None),
     ];
     println!("{:?}", sp_paths);

     let mut sp_builder = ShortpathsBuilder::new(sp_paths);

     let sp_im = sp_builder.build().unwrap();
     println!("{:?}", sp_im);

     // Test find_key
     //let key = sp_im.find_key_for_value("$a/bbbb");
     //println!("{:?}", key);

     //let key = sp_im.find_key_for_value("$a/bbbb".to_string());
     //println!("{:?}", key);

}
