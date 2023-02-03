use std::path::{PathBuf, Component};

use indexmap::IndexMap;

type SP = IndexMap<String, Shortpath>;
type DEPS = Vec<ShortpathDependency>; 

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortpathDependency {
    Alias(String, PathBuf),
    EnvironmentVar(String, PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortpath {
    /// Name of the shortpath
    name: String,
    /// The path to be stored on disk
    entry: PathBuf,
    /// The full file path
    full_path: Option<PathBuf>,
    /// Any dependent shortpaths
    deps: Option<DEPS>,
}

struct Shortpaths {
    paths: SP,
}

impl Shortpath {
    pub fn new<S, P>(name: S, entry: P, full_path: Option<PathBuf>, deps: Option<DEPS>) -> Shortpath
    where
        S: Into<String>,
        P: Into<PathBuf>
    {
        Shortpath { name: name.into(), entry: entry.into(), full_path, deps }
    }
}

impl Shortpaths {
    // TODO: Create empty new construct
    // TODO: Use FromIterator trait extension
    pub fn new(sps: Vec<Shortpath>) -> SP {
        let mut im: SP = indexmap::IndexMap::new();
        sps.into_iter().for_each(|sp| {
            im.insert(sp.name.clone(), sp);
        });
        im
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
            .filter_map(|(key, val)| if val.entry.to_str().unwrap().to_owned() == v { Some(key) } else { None })
            .collect()
    }

    fn find_key_for_value(&'a self, value: V) -> Option<&'a String> {
        let v = value.into();
        self.iter().find_map(|(key, val)| if val.entry.to_str().unwrap().to_owned() == v { Some(key) } else { None })
    }
}

// These functions are used for prettifying the file during export phase

// TODO: Create pure FP functions that return pure values (for testing)
// TODO: Create impure interface that stores the result in the Shortpath struct

// Pure Functions

/// Find the longest possible keyname in the hashmap
pub fn find_longest_keyname(map: &SP) -> String {
    map.into_iter()
       .max_by(|(k1,_), (k2, _)| k1.len().cmp(&k2.len()))
       .unwrap().0.to_owned()
}

/// Convert strings into a vector of characters
pub fn to_str_slice(s: impl Into<String>) -> Vec<char> {
    s.into().chars().collect()
}

/// Get the type of alias
pub fn parse_alias(path: &[char]) -> Option<ShortpathDependency> {
    match path {
        ['$', alias_name @ ..] => {
            let (an, ap) = (alias_name.iter().collect(), PathBuf::from(path.iter().collect::<String>()));
            Some(ShortpathDependency::Alias(an, ap))
        }
        [ '{', '$', 'e', 'n', 'v', ':', alias_name @ .., '}'] => {
            let (an, ap) = (alias_name.iter().collect(), PathBuf::from(path.iter().collect::<String>()));
            Some(ShortpathDependency::EnvironmentVar(an, ap))
        }
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

/// Lookup the shortpath dependency in the shortpaths index map
pub fn parse_shortpath_dependency(dep: ShortpathDependency, sp: &SP) -> (String, String) {
    let mut key_name = String::new();
    let mut key_path = String::new();

    match dep {
        ShortpathDependency::Alias(name, _) => {
            key_path = sp.get(&name)
                .expect(&format!("Could not get key: {}", name))
                .entry.to_str().unwrap().to_owned();
            key_name = name;
        }
        ShortpathDependency::EnvironmentVar(name, _) => {
            match std::env::var(&name) { // Get from environment
                Ok(var) => key_path = var,
                Err(e) => eprintln!("Error in expanding environment variable: ${}", e)
            };
            key_name = name;
        }
    }
    (key_name, key_path)
}



/// Expand the entry path into the full path of the given entry
pub fn expand_full_path(mut sp: Shortpath, shortpaths: &SP) -> String {
    let entry = sp.entry.to_str().unwrap().to_owned();
    let mut output = entry.clone();

    let unwrap_sp_dp = move |dep: &ShortpathDependency| {
        let (key_name, key_path) = parse_shortpath_dependency(dep.to_owned(), shortpaths);
        (key_name,key_path)
    };

    match &sp.deps {
        Some(deps) => { // Expand entry into full_path
            deps.iter().for_each(|dep| {
                let (key_name, key_path) = unwrap_sp_dp(dep);
                //let (key_name, key_path) = parse_shortpath_dependency(dep.to_owned(), shortpaths);
                output = fmt_expand(&output, &key_name, &key_path)
            });
        }
        None => { // Use the entry as the full_path
            sp.full_path = Some(sp.entry);
        }
    };
    return output;
}

// Impure Shortpath Functions

/// Expand the entry path into the full path of the given entry
pub fn sp_pop_full_path(mut sp: Shortpath, shortpaths: &SP) -> String {
    assert!(sp.full_path.is_none());

    let entry = sp.entry.to_str().unwrap().to_owned();
    let mut output = String::new();
    match &sp.deps {
        Some(deps) => { // Expand entry into full_path
            deps.iter().for_each(|dep| {
                let (key_name, key_path) = parse_shortpath_dependency(dep.to_owned(), shortpaths);
                output = fmt_expand(&entry, &key_name, &key_path)
                
            });
        }
        None => { // Use the entry as the full_path
            sp.full_path = Some(sp.entry);
        }
    };
    return output;
}

/// Populate the dependencies of a shortpath
pub fn pop_deps_sp(sp: &mut Shortpath) -> &Shortpath {
    if sp.deps.is_some() {
        return sp; // Return, if already populated
    }
    sp.deps = find_deps(&sp.entry);
    sp
}

// This shouldn't  be done like this
pub fn expand_shortpath(pname: &String, sp_dep: &Option<String>, sp: &SP) -> String {
    let mut output = String::new();
    let path = sp.get(pname).unwrap().entry.to_str().unwrap().to_string();
    match sp_dep {
        // If there's a dependency
        Some(dep) => {
            while let Some(alias_name) = parse_alias(&to_str_slice(dep)) {
                let mut key_name = String::new();
                let mut key_path = String::new();

                // Get the name and path of the dependent shortpath
                match alias_name {
                    ShortpathDependency::Alias(name, _) => {
                        key_name = name;
                        key_path = sp.get(pname).expect(&format!("Could not get key: {}", pname))
                            .entry.to_str().unwrap().to_owned();
                    }
                    ShortpathDependency::EnvironmentVar(name, _) => {
                        key_name = name.clone();
                        match std::env::var(name) { // Get from environment
                            Ok(var) => key_path = var,
                            Err(e) => eprintln!("Error in expanding environment variable: ${}", e)
                        };
                    }
                }

                // Expand the variable path
                let this = format!("${}", key_name);
                let with = key_path;
                output = path.replace(&this, &with);
            }
        }
        // Else, just use the path as is directly
        None => {
            output = sp.get(pname).unwrap().entry.to_str().unwrap().to_string();
        }
    }
    output
}

/// Return the order of the shortpaths to serialize
pub fn expand_shortpaths(dmap: IndexMap<String, Option<String>>, sp: &SP) -> Vec<String> {
    // We want to try out all the keys, then peform the expansion if it is considered valid
    // This will allow us to pull out as many aliases as needed

    // TODO:  This should not be done like before
    // We need to use the indexmap to perform the sort because we will require the key name and expanded path later
    // In order to serialize to disk
    let ordpaths: Vec<String> = dmap.iter().map(|(pname, sp_dep)| {
        expand_shortpath(pname, sp_dep, &sp)
    }).collect();
    ordpaths
}

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
     let sp_paths = vec![
         Shortpath::new("a", "aaaa", None, None),
         Shortpath::new("b", "$a/bbbb", None, None),
         Shortpath::new("c", "$b/cccc", None, None),
         Shortpath::new("d", "$a/dddd", None, None),
     ];
     println!("{:?}", sp_paths);

     let mut sp_im = Shortpaths::new(sp_paths);
     println!("{:?}", sp_im);

     // Test find_key
     let key = sp_im.find_key_for_value("$a/bbbb");
     println!("{:?}", key);

     let key = sp_im.find_key_for_value("$a/bbbb".to_string());
     println!("{:?}", key);

     // Test dependency graph
     //sp_im.iter_mut().for_each(|(_name, sp)| {
         //let deps = find_deps(sp);
         //sp.deps = deps;
     //});
     //gen_deps_graph(&sp_im);
     
}
