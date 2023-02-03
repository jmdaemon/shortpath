use std::path::{PathBuf, Component};

use indexmap::IndexMap;

type SP = IndexMap<String, Shortpath>;
type SPT = ShortpathType;
type DEPS = Vec<SPT>; 

/// The type of shortpath it is
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortpathType {
    Path(String, PathBuf),      // Shortpath Name: Shortpath Path
    AliasPath(String, PathBuf), // Shortpath Name: Shortpath Path
    EnvPath(String, PathBuf),   // Env Var Name: Shortpath Path
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
    pub fn new_env_path(name: impl Into<String>, alias_path: impl Into<PathBuf>) -> ShortpathType {
        ShortpathType::EnvPath(name.into(), alias_path.into())
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
                // Populate the deps of every shortpath
                sp_pop_deps(sp);
            });

            //let spsref = &sps;
            //sps.into_iter().for_each(|(_, sp)| {
                //// Populate the deps of every shortpath
                //sp_pop_deps(sp);
            //});

            //sps.into_iter().for_each(|(_, sp)| {
                //// Populate the full_path field of every shortpath
                //sp_pop_full_path(sp, *spsref);
            //});

            // Return to the user
            return Some(shortpaths.to_owned());
        }
        None
    }
}

// Trait Extensions
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

// These functions are used for prettifying the file during export phase

// TODO: Create pure FP functions that return pure values (for testing)
// TODO: Create impure interface that stores the result in the Shortpath struct

// Pure Functions

// General Purpose
/// Convert strings into a vector of characters
pub fn to_str_slice(s: impl Into<String>) -> Vec<char> {
    s.into().chars().collect()
}

// Shortpaths Specific
// TODO: Generalize this
/// Find the longest possible keyname in the hashmap
pub fn find_longest_keyname(map: &SP) -> String {
    map.into_iter()
       .max_by(|(k1,_), (k2, _)| k1.len().cmp(&k2.len()))
       .unwrap().0.to_owned()
}

/// Get the type of alias
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
        // TODO Parse the Path
        // TODO We can even remove the option wrapping here since we wont have a null value
        _ => { None }
    }
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

pub fn get_shortpath_key_name(sp: SPT) -> Option<String> {
    match sp {
        SPT::Path(name, _) | SPT::AliasPath(name, _) | SPT::EnvPath(name, _) => Some(name),
    }
}

/// Lookup the shortpath dependency in the shortpaths index map
pub fn parse_shortpath_dependency(dep: SPT, sp: &SP) -> (String, String) {
    let mut key_name = String::new();
    let mut key_path = String::new();

    match dep {
        SPT::AliasPath(name, _) => {
            key_path = sp.get(&name)
                .expect(&format!("Could not get key: {}", name))
                .path().to_str().unwrap().to_owned();
            key_name = name;
        }
        SPT::EnvPath(name, _) => {
            match std::env::var(&name) { // Get from environment
                Ok(var) => key_path = var,
                Err(e) => eprintln!("Error in expanding environment variable: ${}", e)
            };
            key_name = name;
        }
        _ => {}
    }
    (key_name, key_path)
}

/// Expand the entry path into the full path of the given entry
pub fn expand_full_path(entry: String, sp: &Shortpath, unwrap_sp_dp: impl Fn(&SPT) -> (String, String)) -> String {
    let mut output = entry.clone();
    match &sp.deps {
        Some(deps) => // Expand entry into full_path
            deps.iter().for_each(|dep| {
                let (key_name, key_path) = unwrap_sp_dp(dep);
                output = fmt_expand(&output, &key_name, &key_path);
            }),
        None => output = entry // Use the entry as the full_path
    };
    return output;
}

// Impure Shortpath Functions

/// Populate the dependencies of a shortpath
pub fn sp_pop_deps(sp: &mut Shortpath) {
    if sp.deps.is_some() { return; }
    sp.deps = find_deps(&sp.path());
}

/// Populate the full_path field of a shortpath
pub fn sp_pop_full_path(sp: &mut Shortpath, shortpaths: &SP) {
    assert!(sp.full_path.is_none());

    let unwrap_sp_dp = move |dep: &SPT| {
        let (key_name, key_path) = parse_shortpath_dependency(dep.to_owned(), shortpaths);
        (key_name,key_path)
    };

    let entry = sp.path().to_str().unwrap().to_owned();
    let output = expand_full_path(entry, &sp, unwrap_sp_dp);
    sp.full_path = Some(PathBuf::from(output));
}

///// Expand a single shortpath
//pub fn expand_shortpath(name: String, sp: &SP) -> PathBuf {
//    let shortpath = sp.get(&name).unwrap().to_owned();
//    shortpath.full_path.unwrap()
//}

///// Expand all shortpaths in the index map
//pub fn expand_shortpaths(sp: &mut SP) {
//    sp.into_iter().for_each(|(_key, shortpath)| {
//        sp_pop_full_path(shortpath, &sp.to_owned());
//    });
//}

// Now that we have the dependency vector, we're going to loop through and generate the graph for the dep tree
// We want to be able to then use this tree to order

/* What do we want?
 * We want a data structure with the following:
 * - Constant space-time key indexing
 * - Custom ordererings (Sort by group/dependencies first, Chronological (Keep the order they are now)
 * - Least intrusive paths
 * - Serialization with tab alignment
 * - Orderable HashMap with Tree implementation
 * We want to
 *  - Group by dependency
 */

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
     let key = sp_im.find_key_for_value("$a/bbbb");
     println!("{:?}", key);

     let key = sp_im.find_key_for_value("$a/bbbb".to_string());
     println!("{:?}", key);

     // Populate
     // If we make a builder then we can populate the fields here
}
