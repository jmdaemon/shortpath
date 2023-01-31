use std::path::PathBuf;

use indexmap::{IndexMap, indexmap};

/* IndexMap
 * - Remembers order chronologically
 * - Allows for custom sorting
 */

enum ShortpathDependency {
    Shortpath(Shortpath),
    EnvironmentVar(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Shortpath {
    full_path: PathBuf,
    alias_name: Option<String>,
    alias_path: Option<PathBuf>,
}

struct Shortpaths {
    paths: IndexMap<String, Shortpath>,
    path_deps: IndexMap<String, ShortpathDependency>

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
    let sp = Shortpath { full_path: PathBuf::from("aaaa"), alias_name: None, alias_path: None};
    let im: IndexMap<String, Shortpath> = indexmap! {
        "aaaa".to_owned() => sp,
    };

    let key = im.find_keys_for_value(&im.get("aaaa").unwrap());
    println!("{:?}", key);
}
