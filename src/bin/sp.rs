use std::path::{PathBuf, Component};
use std::ffi::OsStr;

use indexmap::{IndexMap, indexmap};
use petgraph::dot::{Config, Dot};

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

// This doesn't make sense
// We won't be able to resolve the full path until runtime
// Shortpath in and of itself does not store its own alias name & path?
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortpath {
    //full_path: PathBuf,
    //entry: ShortpathDependency,
    name: String,
    //entry: ShortpathDependency,
    entry: PathBuf,
    full_path: Option<PathBuf>,
    deps: Option<DEPS>,
}
// This doesn't make much sense necessarily?
// Should a shortpath contain multiple aliases?

struct Shortpaths {
    paths: SP,
}

// TODO: Shortpath Creation API
// - We need to be able to create shortpaths easily
// - It needs to make sense
// - Should support our current ops
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
        //sps.into_iter().map(|sp| {
        sps.into_iter().for_each(|sp| {
            //(sp.full_path.clone(), sp)
            //im.insert(sp.full_path.clone(), sp);
            //im.insert(sp.full_path.clone(), sp);
            //let sp = Shortpath::new(spd, None, None);
            im.insert(sp.name.clone(), sp);
        });
        im
    }
}

// Trait Extensions
pub trait FindKeyIndexMapExt<'a, K,V: PartialEq + Eq> {
    /// Get keys from value of IndexMap
    fn find_keys_for_value(&'a self, value: V) -> Vec<&'a K>;

    /// Get key from value of IndexMap
    fn find_key_for_value(&'a self, value: V) -> Option<&'a K>;
}

//pub trait FindKeyIndexMapStrExt<'a, K> {
    //fn find_keys_for_value(&'a self, value: &str) -> Vec<&'a K>;

    //fn find_key_for_value(&'a self, value: &str) -> Option<&'a K>;
//}

// Generic Implementation
impl<'a, K, V> FindKeyIndexMapExt<'a, K,V> for IndexMap<K,V>
where V: PartialEq + Eq
{
    fn find_keys_for_value(&'a self, value: V) -> Vec<&'a K> {
        self.into_iter()
            .filter_map(|(key, val)| if *val == value { Some(key) } else { None })
            .collect()
    }

    fn find_key_for_value(&'a self, value: V) -> Option<&'a K> {
        self.iter().find_map(|(key, val)| if *val == value { Some(key) } else { None })
    }

}

// Lookup using just the path part
//impl<'a, K, V> FindKeyIndexMapExt<'a, K,V> for IndexMap<K,V>
//where V: Eq
//{
    //fn find_keys_for_value(&'a self, value: &V) -> Vec<&'a K> {
        //self.into_iter()
            //.filter_map(|(key, val)| if val == value { Some(key) } else { None })
            //.collect()
    //}

    //fn find_key_for_value(&'a self, value: &V) -> Option<&'a K> {
        //self.iter().find_map(|(key, val)| if val == value { Some(key) } else { None })
    //}

//}

//impl<'a> FindKeyIndexMapStrExt<'a, String> for IndexMap<String,Shortpath>
impl<'a> FindKeyIndexMapExt<'a, String, &str> for IndexMap<String,Shortpath>
{
    fn find_keys_for_value(&'a self, value: &str) -> Vec<&'a String> {
        self.into_iter()
            .filter_map(|(key, val)| if val.entry.to_str().unwrap().to_owned() == value { Some(key) } else { None })
            .collect()
    }

    fn find_key_for_value(&'a self, value: &str) -> Option<&'a String> {
        //self.iter().find_map(|(key, val)| if val == value { Some(key) } else { None })
        self.iter().find_map(|(key, val)| if val.entry.to_str().unwrap().to_owned() == value { Some(key) } else { None })
    }
}

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

pub fn to_str_slice(s: &OsStr) -> Vec<char> {
    let spath = s.to_str().unwrap().to_string();
    let chars: Vec<char> = spath.chars().collect();
    chars
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
pub fn gen_deps_tree(sp: &Shortpath, _map: &SP) -> Option<DEPS> {
    // Return, if already populated
    if let Some(deps) = &sp.deps {
        return Some(deps.to_owned())
    }

    if let Some(full_path) = &sp.full_path {
        let deps: DEPS =
        full_path.components().into_iter().filter_map(|p| {
            if let Component::Normal(ostrpath) = p {
                return parse_alias(&to_str_slice(ostrpath));
            }
            return None
        }).collect();
        return Some(deps)
    }
    None
}

/// Create a dependency graph from the vector of dependencies
pub fn gen_deps_graph(deps: &DEPS, sp: &SP) {
    //let tree = id_tree::
    //let mut fr = fr::<String>();
    let mut depgraph = petgraph::graph::DiGraph::new();
    sp.into_iter().for_each(|(name, deps)| {
        match &deps.deps {
            Some(sp_deps) => {
                // For every dependency
                // Get the name and path
                // Get the alias to which the damn thing is connected to
                // Add the node from: name -> alias
                sp_deps.iter().for_each(|spd| {
                    match spd {
                        ShortpathDependency::Alias(to, path) => {
                            // Create name, to nodes
                            // Attach name to node
                            //let tree = tr(name.to_owned()) / tr(to.to_owned()); 
                            //fr.push_back(tree);
                            //let src = petgraph::graph::Node::from(name);
                            let src = depgraph.add_node(name);
                            let dest = depgraph.add_node(to);
                            depgraph.add_edge(src, dest, path);
                            //name
                        }
                        ShortpathDependency::EnvironmentVar(to, path) => {}
                    }
                });
            }
            None => {
                // Add the node to our graph anyways

            }
        }
    });
    // Once the graph is initialized, we want to be able to traverse it
    // We will use
    //let a = fr.to_string();
    //let b = trees::tree::Tree::from(a);
    //let mst = trees::Tree::from_tuple(a);
    
    //let asf = petgraph::algo::min_spanning_tree(depgraph);

    //let mut space = petgraph::algo::DfsSpace::new(&depgraph);
    //let sorted = petgraph::algo::toposort(&depgraph, Some(&mut space));
    //let indices = sorted.unwrap();

    println!("{:?}", Dot::with_config(&depgraph, &[Config::EdgeIndexLabel]));
    
    // First add all the nodes to the graph
    //sp.iter();

    // For every shortpath
    // Get the name
    // Create a node
    // For every dependency
    // 

    // We ha ve the following paths
    // /home/jmd/test/appthing
    // How do we create the graph of dependencies from this?
    //deps.iter().for_each(|sd| {
        ////let (alias, path): (String, PathBuf) = sd;
        //match sd {
            //ShortpathDependency::Alias(name, path) => {
                //// Attach to this node
                ////sp.find_key_for_value(path);
                //// 

            //}
            //ShortpathDependency::EnvironmentVar(name, _) => {
                
            //}
        //}

    //});

    // Then determine the edges to which they all connect
    //let mut p 
    //deps.iter().for_each(
        //f
        //);

    // Once we determine how they're all connectedj
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
     let sp_a = Shortpath::new("a", "aaaa", None, None);
     let sp_b = Shortpath::new("b", "$a/bbbb", None, None);

     let sp_paths = vec![sp_a, sp_b.clone()];
    println!("{:?}", sp_paths);

     let sp_im = Shortpaths::new(sp_paths);
    println!("{:?}", sp_im);

    //let key = sp_im.find_keys_for_value(sp_im.get("$a/bbbb").unwrap());
    //let key = sp_im.find_key_for_value(sp_im.get("$a/bbbb").unwrap());

    //let val = sp_im.get("$a/bbbb");
    //println!("{:?}", val);

    //let key = sp_im.find_key_for_value(&sp_b.clone());
    //println!("{:?}", key);

    let key = sp_im.find_key_for_value("$a/bbbb");
    println!("{:?}", key);

     
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