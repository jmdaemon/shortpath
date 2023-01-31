use std::path::{PathBuf, Component};

use indexmap::{IndexMap, indexmap};

/* IndexMap
 * - Remembers order chronologically
 * - Allows for custom sorting
 */

#[derive(Debug, PartialEq, Eq)]
pub enum ShortpathDependency {
    //Shortpath(Shortpath),
    //Alias(Option<String>, Option<PathBuf>),
    Alias(String, PathBuf),
    EnvironmentVar(String, PathBuf),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Shortpath {
    full_path: PathBuf,
    deps: Option<Vec<ShortpathDependency>>,
    //alias_name: Option<String>,
    //alias_path: Option<PathBuf>,
}
// This doesn't make much sense necessarily?
// Should a shortpath contain multiple aliases?

struct Shortpaths {
    paths: IndexMap<String, Shortpath>,
    //path_deps: IndexMap<String, ShortpathDependency>
}

pub trait FindKeyIndexMapExt<'a, K,V: Eq> {
    /// Get keys from value of IndexMap
    fn find_keys_for_value(&'a self, value: &V) -> Vec<&'a K>;

    /// Get key from value of IndexMap
    fn find_key_for_value(&'a self, value: &V) -> Option<&'a K>;
}

impl<'a, K, V> FindKeyIndexMapExt<'a, K,V> for IndexMap<K,V>
where V: Eq
{
    fn find_keys_for_value(&'a self, value: &V) -> Vec<&'a K> {
        self.into_iter()
            .filter_map(|(key, val)| if val == value { Some(key) } else { None })
            .collect()
    }

    fn find_key_for_value(&'a self, value: &V) -> Option<&'a K> {
        self.iter().find_map(|(key, val)| if val == value { Some(key) } else { None })
    }

}

type SP = IndexMap<String, Shortpath>;

// These functions are used for prettifying the file during export phase

/// Find the longest possible keyname in the hashmap
pub fn find_longest_keyname(map: &SP) -> String {
    let mut max = String::new();
    map.into_iter().for_each(|(k,_)| {
        if k.len() > max.len() {
            max = k.to_owned()
        }
    });
    max
}

pub fn get_padding_len(map: &SP) -> usize {
    let max = find_longest_keyname(map);
    max.len()
}

/// Generate the dependencies the shortpath requires
/// We assume the deps are empty, and that we must populate the dependency
pub fn gen_deps_tree(sp: Shortpath, _map: &SP) -> Option<Vec<ShortpathDependency>> {
    // Return, if already populated
    if let Some(deps) = sp.deps {
        return Some(deps)
    }

    // Else attempt to determine shortpaths
    // Pattern match two syntaxes, 
    // ${env:}: EnvironmentVar
    // $: Shortpaths
    sp.full_path.components().into_iter().for_each(|p| {
        if let Component::Normal(ostrpath) = p {
            // Match and check if the path is equal to something
            // Use the @ .. syntax

            // If there's a match
            // Try to do this part in a function that returns the enum variant, then just match on that instead
            match ostrpath.to_str().unwrap() {
                "${env:}" => {
                    // Get the name of the key,
                    // Get the environment variable path
                    // Make the dependency
                    // Collect in deps
                },
                "$" => {
                    // Get the name of the key,
                    // Get the shortpath variable path
                    // Make the dependency
                    // Collect in deps
                }
                _ => {} // Skip
            };
        }

    });

    //if let Some(deps) = sp.deps {
        //deps.iter().for_each(|d|
            //match d {
                //ShortpathDependency::Alias(name, path) => {

                //},
                //ShortpathDependency::EnvironmentVar(var) => {}
            //});
    //}
    None
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
    //let alias = ShortpathDependency::Alias(None, None);
    let alias = ShortpathDependency::Alias("bbbb".to_owned(), PathBuf::from("bbbb"));
    let sp = Shortpath { full_path: PathBuf::from("aaaa"), deps: Some(vec![alias]) };
    let im: IndexMap<String, Shortpath> = indexmap! {
        "aaaa".to_owned() => sp,
    };

    let key = im.find_keys_for_value(&im.get("aaaa").unwrap());
    println!("{:?}", key);
}
