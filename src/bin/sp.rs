use std::path::{PathBuf, Component};
use std::ffi::OsStr;

use indexmap::IndexMap;
use petgraph::Graph;
use petgraph::dot::{Config, Dot};
use petgraph::stable_graph::NodeIndex;

/* IndexMap
 * - Remembers order chronologically
 * - Allows for custom sorting
 */

type SP = IndexMap<String, Shortpath>;
type DEPS = Vec<ShortpathDependency>; 

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortpathDependency {
    Alias(String, PathBuf),
    EnvironmentVar(String, PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortpath {
    name: String,
    entry: PathBuf,
    full_path: Option<PathBuf>,
    deps: Option<DEPS>,
}
// This doesn't make much sense necessarily?
// Should a shortpath contain multiple aliases?

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

/// Find the longest possible keyname in the hashmap
pub fn find_longest_keyname(map: &SP) -> String {
    map.into_iter()
       .max_by(|(k1,_), (k2, _)| k1.len().cmp(&k2.len()))
       .unwrap().0.to_owned()
}

//pub fn to_str_slice(s: &OsStr) -> Vec<char> {
pub fn to_str_slice(s: impl Into<String>) -> Vec<char> {
    s.into().chars().collect()
}

/// Get the type of alias
pub fn parse_alias(path: &[char]) -> Option<ShortpathDependency> {
    // Pattern matches two syntaxes
    // ${env:}: EnvironmentVar
    // $: Shortpaths
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

/// Generate the dependencies the shortpath requires
/// We assume the deps are empty, and that we must populate the dependency
pub fn gen_deps_tree(sp: &Shortpath) -> Option<DEPS> {
    // Return, if already populated
    if let Some(deps) = &sp.deps {
        return Some(deps.to_owned())
    }

    let deps: DEPS =
    sp.entry.components().into_iter().filter_map(|p| {
        if let Component::Normal(ostrpath) = p {
            return parse_alias(&to_str_slice(ostrpath.to_string_lossy()));
        }
        return None
    }).collect();
    return Some(deps)
}

/// Create a dependency graph from the vector of dependencies
pub fn gen_deps_graph(sp: &SP) {
    // LinkedList
    // 1. Create a Vec<(String), Option (String)> aliases
    // 2. For every elem in vec
    //      if let Some ()
    let spds: Vec<(String, Option<String>)> = vec![];
    
    // Key Name: Optional Key Alias
    let dmap: IndexMap<String, Option<String>> = spds.into_iter().collect();
    // We'll need to continually match and expand this in order
    //let dmap = spds.into_iter().for_each(|(from, to)| {
        ////if let Some(dest) = to {
        ////}
    //});
    
    let mut depgraph = petgraph::graph::DiGraph::new();
    sp.into_iter().for_each(|(name, deps)| {
        match &deps.deps {
            Some(sp_deps) => {
                sp_deps.iter().for_each(|spd| {
                    match spd {
                        ShortpathDependency::Alias(to, path) => {
                            let src = depgraph.add_node(name);
                            let dest = depgraph.add_node(to);
                            depgraph.add_edge(src, dest, path.to_owned());
                            //depgraph.extend_with_edges(&[
                                //(src, dest),
                            //]);
                        }
                        ShortpathDependency::EnvironmentVar(to, path) => {
                            let src = depgraph.add_node(name);
                            let dest = depgraph.add_node(to);
                            depgraph.add_edge(src, dest, path.to_owned());
                        }
                    }
                });
            }
            None => {
                // Add the node to our graph anyways
                depgraph.add_node(name);
            }
        }
    });

    println!("{:?}", Dot::with_config(&depgraph, &[Config::EdgeIndexLabel]));
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

pub fn sort_graph(depgraph: Graph<&String, &PathBuf>) -> Vec<NodeIndex> {
    let mut space = petgraph::algo::DfsSpace::new(&depgraph);
    let sorted = petgraph::algo::toposort(&depgraph, Some(&mut space));
    let indices = sorted.unwrap();
    indices
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
     sp_im.iter_mut().for_each(|(_name, sp)| {
         let deps = gen_deps_tree(sp);
         sp.deps = deps;
     });
     gen_deps_graph(&sp_im);
     
    //let sp = Shortpath::new("aaaa", ShortpathDependency::Alias("aaaa", ));
    //let alias = ShortpathDependency::Alias(None, None);
    //let alias = ShortpathDependency::Alias("bbbb".to_owned(), PathBuf::from("bbbb"));
    //let sp = Shortpath { full_path: PathBuf::from("$bbbb/aaaa"), deps: Some(vec![alias]) };

    //let im: SP = indexmap! {
        //// Path     : Shortpath
        //"$bbbb/aaaa".to_owned() => sp.clone(),
    //};

    // Test find_keys
    //let key = im.find_keys_for_value(&im.get("$bbbb/aaaa").unwrap());
    //println!("{:?}", key);

    // Test the graph
    //let spds = vec![
        //ShortpathDependency::Alias("a".to_owned(), PathBuf::from("aaaa")),
        //ShortpathDependency::Alias("b".to_owned(), PathBuf::from("$a/bbbb")),
        //ShortpathDependency::Alias("c".to_owned(), PathBuf::from("$a/cccc")),
        //ShortpathDependency::Alias("d".to_owned(), PathBuf::from("$c/dddd")),
    //];
    
    //let sim: SP = spds.iter().for_each(|spd| {
        //// Expand path

    //});

    //let deps = gen_deps_tree(&sp, &im).unwrap();
    //gen_deps_graph(&deps, &im);
}
