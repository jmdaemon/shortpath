use crate::export::{Export, get_exporter};
use crate::helpers::{
    find_by_matching_path,
    find_paths,
    to_str_slice,
};
use std::fmt::format;
use std::path::Components;
use std::{
    path::{Path, PathBuf, Component},
    cmp::Ordering,
};

use indexmap::IndexMap;
#[allow(unused_imports)]
use itertools::Itertools;
use log::{trace, info};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use walkdir::DirEntry;

// Data Types
pub type SP = IndexMap<String, Shortpath>;
pub type SPD = ShortpathDependency;
pub type DEPS = Vec<SPD>; 

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortpathVariant {
    Independent,
    Alias,
    Environment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortpathDependency(ShortpathVariant, String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortpath {
    pub path: PathBuf,
    pub full_path: Option<PathBuf>,
    pub deps: DEPS,
}

pub struct ShortpathsBuilder {
    paths: Option<SP>,
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
        let sp = Shortpath::new(PathBuf::from(path), None, vec![]);
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
    pub fn new(path: PathBuf, full_path: Option<PathBuf>, deps: DEPS) -> Shortpath {
        Shortpath { path, full_path, deps }
    }
}

impl ShortpathsBuilder {
    // TODO: Use FromIterator trait extension
    pub fn new(sp: SP) -> ShortpathsBuilder  {
        ShortpathsBuilder { paths: Some(sp) }
    }

    pub fn build(&mut self) -> Option<SP> {
        if let Some(shortpaths) = &mut self.paths {
            let shortpaths = populate_shortpaths(shortpaths);
            return Some(shortpaths);
        }
        None
    }
}

// Trait Extensions
// We need these for remove_hook, move_hook to check shortpath configs
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
            .filter_map(|(key, val)| if val.path.to_str().unwrap() == v { Some(key) } else { None })
            .collect()
    }

    fn find_key_for_value(&'a self, value: V) -> Option<&'a String> {
        let v = value.into();
        self.iter().find_map(|(key, val)| if val.path.to_str().unwrap() == v { Some(key) } else { None })
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

pub fn sort_shortpaths(shortpaths: SP) -> SP {
    shortpaths.sorted_by(|_, v1, _, v2| {
        v1.cmp(v2)
    }).collect()
}

// Input Parsing

// Populate Shortpaths
pub fn populate_dependencies(shortpaths: &mut SP) -> SP {
    let c = shortpaths.clone();
    let shortpaths: SP = shortpaths.into_iter().map(|(k, sp)| {
        let deps = find_deps(&sp.path, false, &c);
        trace!("Dependencies for Shortpath: {} : {:?}", k, deps);
        sp.deps = deps;
        (k.to_owned(), sp.to_owned())
    }).collect();
    shortpaths
}

pub fn populate_expanded_paths(shortpaths: &mut SP) -> SP {
    let shortpaths_copy = shortpaths.clone();
    // Expand to full_path
    let shortpaths: SP = shortpaths.into_iter().map(|(k, sp)| {
        let full_path = expand_shortpath(sp, &shortpaths_copy);
        sp.full_path = Some(full_path);
        (k.to_owned(), sp.to_owned())
    }).collect();
    shortpaths
}

pub fn populate_shortpaths(shortpaths: &mut SP) -> SP {
    let mut shortpaths = populate_dependencies(shortpaths);
    populate_expanded_paths(&mut shortpaths)
}

/** Parse a Shortpath entry, and returns any dependencies */
pub fn get_shortpath_dependency(path: &[char]) -> Option<SPD> {
//pub fn get_shortpath_dependency(path: &[char]) -> SPD {
    // Closures
    let to_string = |slice: &[char]| { slice.iter().collect() };
    let _env_prefix = to_str_slice("$env:");
    let is_not_empty = |path: &[char]| { !path.is_empty() };
    let to_spd = |variant: ShortpathVariant, dependency_name: String| { ShortpathDependency(variant, dependency_name) };

    match (is_not_empty(path), path) {
        (true, ['$', alias_name @ ..])                  => Some(to_spd(ShortpathVariant::Alias, to_string(alias_name))),
        (true, [ _env_prefix, env_var_name @ .., '}'])  => Some(to_spd(ShortpathVariant::Environment, to_string(env_var_name))),
        (true, [ normal_path @ ..])                     => Some(to_spd(ShortpathVariant::Independent, to_string(normal_path))),
        (false, _)                                      => None
    }
}

/** Return the type of a shortpath entry */
//pub fn get_shortpath_type(path: &[char]) -> Option<ShortpathVariant> {
//pub fn get_shortpath_type(comp: Component) -> Option<ShortpathVariant> {
pub fn get_shortpath_type(comp: &[char]) -> Option<ShortpathVariant> {
    //let to_string = |slice: &[char]| { slice.iter().collect() };
    let _env_prefix = to_str_slice("$env:");
    let is_not_empty = |path: &[char]| { !path.is_empty() };
    //let to_spd = |variant: ShortpathVariant, dependency_name: String| { ShortpathDependency(variant, dependency_name) };

    
    //comp.as_os_str().to_str().unwrap().to_string().chars())
    //match (is_not_empty(path), path) {
    //let is_not_empty = |comp: Component| { !comp.as_os_str().is_empty() };
    //let path = to_str_slice(&comp.as_os_str().to_str().unwrap();
    //match (is_not_empty(), &path) {
    match (is_not_empty(comp), comp) {
        (true, ['$', alias_name @ ..])                  => Some(ShortpathVariant::Alias),
        (true, [ _env_prefix, env_var_name @ .., '}'])  => Some(ShortpathVariant::Environment),
        (true, [ normal_path @ ..])                     => Some(ShortpathVariant::Independent),
        (false, [])                                     => None,
        (false, _)                                      => None
    }
}


/** Find the dependencies for a given shortpath
  *
  * Algorithm:
  * For every component in the path.components()
  *     If component is a valid shortpath dependency, add it to our array.
  *
  * For every shortpath dependency in our array
  *     If any shortpath dependency is another shortpath dependency
  *         Recursively yield the next dependency, and add it to our array
  * At the end, merge the two vectors together into one vector */
pub fn find_deps(entry: &Path, in_find_nested_mode: bool, shortpaths: &SP) -> DEPS {
    let mut dependencies: Vec<ShortpathDependency> = Vec::new();

    // For every component in the path.components()
    for comp in entry.components() {
        trace!("entry: {:#?}", entry.display());
        trace!("comp: {:?}", comp);
        if let Component::Normal(osstr_path) = comp {
            // If component is a valid shortpath dependency
            let path_comp_slice = to_str_slice(osstr_path.to_string_lossy());
            let dep = get_shortpath_dependency(&path_comp_slice);

            match dep {
                Some(dep_variant) => {
                    // Add it to our array.
                    //if in_find_nested_mode == false {
                    if !in_find_nested_mode {
                        //dependencies.push(dep_variant.clone());
                    }
                    trace!("dependencies: {:?}", &dependencies);
                    match dep_variant.0 {
                        ShortpathVariant::Independent => { }
                        ShortpathVariant::Alias => {
                            dependencies.push(dep_variant.clone());
                        }
                        ShortpathVariant::Environment => {
                            dependencies.push(dep_variant.clone());
                        }
                    };
                }
                None => {
                    // Otherwise, end our loop
                    break;
                }
            }
        }
    }
    println!("dependencies: {:?}", &dependencies);
    dependencies
    /*
    if dependencies.is_empty() {
        return dependencies;
    }

    // Nested Dependencies
    let is_nested_dep = |dep: &ShortpathDependency, shortpaths: &SP| {
        shortpaths.get(&dep.1).is_some()
    };

    let mut nested_deps: Vec<ShortpathDependency> = Vec::new();

    // For every shortpath dependency in our array
    for dep in dependencies.clone() {
        // If not a nested dependency
        if !is_nested_dep(&dep, shortpaths) {
            break; // Exit
        }

        //if in_find_nested_mode == false {
        //if !in_find_nested_mode {
            //in_find_nested_mode = true;
        //}

        // Recursively yield the next dependency
        let nested_dep = shortpaths.get(&dep.1).unwrap();
        trace!("nested_dep: {}", nested_dep.path.display());
        let entry = &nested_dep.path;
        nested_deps.append(&mut find_deps(entry, true, shortpaths));
    }
    dependencies.append(&mut nested_deps); // Flatten
    trace!("dependencies: {:?}", dependencies);
    dependencies
    */
}

// Parses the alias, and only alias variant of a string
pub fn parse_alias(comp: &str) -> Option<String> {
    let copy = comp.to_string();
    if copy.starts_with('$') {
        Some(copy.split_at(1).1.to_owned())
        //copy.chars()
    } else {
        None
    }
}

pub fn to_string(comp: &Component) -> String {
    comp.as_os_str().to_str().unwrap().to_string()
}

pub fn str_join_path(s1: &str, s2: &str) -> PathBuf {
    let (p1,p2) = (PathBuf::from(s1), PathBuf::from(s2));
    p1.join(p2)
}

/**
  * Expand shortpath variants at runtime
  * 
  * The shortpath dependencies must be populated before this is run. When they are
  * populated, both their name and path values are stored in the enum variant which
  * is accessed here without the hashmap.
  */
pub fn expand_shortpath(sp: &Shortpath, shortpaths: &SP) -> PathBuf {
    let mut entry = sp.path.to_owned();

    // For every component: ($a, bbbb)
    //  If shortpath_type == alias
    //      get expanded = expand_shortpath with alias path
    //      // let expanded_path = PathBuf::from(expanded);
    //      // let joined = 
    //      let output = expanded
    //      Do expansion
    //  If shortpath_type == environment
    //      Do expansion
    //  Else
    //      break;
    //      
    
    //pub fn f(mut entry: PathBuf, shortpaths: &SP) -> Option<PathBuf> {
    //pub fn f(mut entry: String, components: Components, shortpaths: &SP) -> Option<String> {

    // (alias_path='$c/dddd', entry='$c/dddd', sp)
    // -> $c
    //    -> alias
    //    -> '$b/cccc'
    //    -> ('$b/cccc', '$b/cccc', sp)
    //       -> ('$b/cccc', '$b/cccc', sp)
    //       -> $b
    //       -> alias
    //       -> '$a/bbbb'
    //       -> ('$a/bbbb', '$a/bbbb', sp)
    //          -> $a
    //          -> alias
    //          -> 'aaaa'
    //          -> ('aaaa', 'aaaa', sp)
    //             -> None 
    //             -> Return 'aaaa'
    //          -> Return expand_path(entry='$a/bbbb', alias_name='a', alias_path='aaaa')
    //       -> Return expand_path(entry='$b/cccc', alias_name='b', alias_path='aaaa/bbbb')
    //    -> Return expand_path(entry='$c/dddd', alias_name='c', alias_path='aaaa/bbbb/cccc')
    // -> Store entry, send as PathBuf
    //pub fn f(mut alias_path: String, entry: PathBuf, alias_name, shortpaths: &SP) -> Option<String> {
    pub fn f(mut alias_path: String, entry: PathBuf, alias_name: String, shortpaths: &SP) -> String {
        const msg: &str = "Summary";
        dbg!(msg);
        dbg!(&alias_path);
        dbg!(&entry);
        dbg!(&alias_name);
        //let mut output = entry.to_owned();
        //let mut output: Option<String> = None;
        //let mut output: String = None;
        let mut output = String::new();
        //for comp in components {
        //for comp in entry.components() {
        if let Some(comp) = entry.components().next() {
            let comp_slice = to_str_slice(comp.as_os_str().to_str().unwrap());
            let shortpath_type = get_shortpath_type(&comp_slice);

            //match shortpath_type {
            if let Some(shortpath_variant) = shortpath_type {
                //Some(shortpath_variant) => {
                match shortpath_variant {
                    ShortpathVariant::Alias       => {

                        //let alias_name = to_string(&comp);

                        //let name = parse_alias(&alias_name).unwrap();
                        /*
                        let name = if alias_name.is_empty() {
                            //parse_alias(&alias_name).unwrap()
                            to_string(&comp)
                        } else {
                            parse_alias(&alias_name).unwrap()
                        };
                        */

                        /*
                        let name_maybe = parse_alias(&alias_name);
                        dbg!(&name_maybe);
                        let mut name = String::new();
                        if let Some(non_null_name) = name_maybe {
                            name = non_null_name;
                            dbg!(&name);
                        } else {
                            name = parse_alias(&to_string(&comp)).unwrap();
                            //return alias_path;
                        }
                        */

                        // String::new()
                        // $c
                        // $c

                        // If the name is empty
                        dbg!("Start of Name");
                        let name = if alias_name.is_empty() {
                            // Then parse the current path component
                            dbg!("Executing name -> parse_alias first branch");
                            //parse_alias(&to_string(&comp)).unwrap()
                            let alias_name = parse_alias(&to_string(&comp)).unwrap();
                            let sp = shortpaths.get(&alias_name).unwrap();
                            let out = sp.path.to_str().unwrap().to_string();
                            //let out = format!("${}", out);
                            dbg!(&out);
                            out

                        } else {
                            // If not, then if parse alias is 
                            if let Some(parsed) = parse_alias(&alias_name) {

                                dbg!(&parsed);
                                let pbuf = PathBuf::from(parsed);
                                let alias_name = to_string(&pbuf.components().next().unwrap());

                                //let alias_name = format!("${}", parsed);
                                dbg!("Executing name -> parse_alias if branch");

                                //let sp = shortpaths.get(&parsed).unwrap();
                                let sp = shortpaths.get(&alias_name).unwrap();
                                //let sp = shortpaths.find_key_for_value(&alias_name).unwrap();
                                dbg!(&alias_name);
                                let out = sp.path.to_str().unwrap().to_string();

                                //let out = sp.to_owned();
                                //let out = format!("${}", out);
                                dbg!(&out);
                                out
                                //parsed
                            } else {
                                dbg!("Executing name -> parse_alias else branch");
                                //alias_name
                                //let alias_name = parse_alias(&to_string(&comp)).unwrap();

                                //let sp = shortpaths.get(&alias_name).unwrap();
                                //let out = sp.path.to_str().unwrap().to_string();
                                //dbg!(&out);
                                //out
                                String::new()
                            }
                            //else {
                                //let sp = shortpaths.get(&alias_name).unwrap();
                                //sp.path.to_str().unwrap().to_string()
                                //alias_name
                            //}
                        };
                        dbg!("End of Name");

                        //let sp = shortpaths.get(&name).unwrap();
                        //let path = &sp.path;
                        //let expanded = expand_path(entry.to_str().unwrap(), &name, path.to_str().unwrap());
                        //let expanded = expand_path(&alias_path, &name, &alias_path);
                        //output = expand_path(&alias_path, &name, &alias_path);
                        //let expanded = expand_path(&alias_path, &name, &alias_path);
                        if name.is_empty() {
                            return alias_path;
                        } 

                        let n1 = name.clone();
                        let ap1 = alias_path.clone();

                        //let expanded = expand_path(&alias_path, name, &alias_path);
                        let expanded = expand_path(&alias_path, &n1, &ap1);
                        //return expanded
                        //return f(expanded, PathBuf::from(output), name, shortpaths);
                        output = expanded.clone();
                        //return f(expanded, PathBuf::from(output), name, shortpaths);
                        let result = f(expanded, PathBuf::from(output), name, shortpaths);
                        let other_msg = format!("Returned from f with : {}", &result);
                        dbg!(other_msg);
                        return result;

                        //let output = f(expanded, PathBuf::from(output), name, shortpaths);
                        //return output;

                        //let expanded = expand_path(entry.to_str().unwrap(), &name, path.to_str().unwrap());
                        //dbg!(&name); dbg!(&sp); dbg!(&path); dbg!(&expanded);
                        //dbg!("{name}, {:?}, {}, {expanded}", sp, path.display());
                        //output = Some(expanded);
                        //output
                        //output = Some(expanded);
                        //output = expanded;
                        //return Some(output);
                    }
                    ShortpathVariant::Independent => {
                        let path = to_string(&comp);
                        return path;
                        //return Some(output);
                        //return Some(output);
                        //let path = to_string(&comp);
                        //dbg!(&path);

                        /*
                        let leftover = to_string(&comp);
                        let path = str_join_path(&output, &leftover);
                        let str_path = path.to_str().unwrap();
                        dbg!(&str_path);

                        //let name = shortpaths.find_key_for_value(&path).unwrap();
                        //let name = shortpaths.find_key_for_value(&path);
                        let name = shortpaths.find_key_for_value(str_path);
                        dbg!(&name);
                        break;
                        if let Some(name) = name {
                            //let expanded = expand_path(entry.to_str().unwrap(), name, &path);
                            let expanded = expand_path(entry.to_str().unwrap(), name, str_path);
                            dbg!(&expanded);
                            output = expanded;
                        } else {
                            break;
                        }
                        */
                    }
                    ShortpathVariant::Environment => {
                        return output;
                        //return Some(output);
                    }
                }
            }
            //None => { }
            //None => { break; }
        //}
        }
    output
        //Some(output)
    }
    // We need to parse the aliases somehow
    // From '$b/cccc' we need to pull out:
    //      - alias_name: b
    //      - alias_path: aaaa
    let mut alias_path = entry.to_str().unwrap().to_string();
    let str_path = f(alias_path, entry, String::new(), shortpaths);
    PathBuf::from(str_path)

    //let mut entry_string = entry.to_str().unwrap().to_string();
    //let alias_path = parse_alias(&entry_string);

    //if let Some(alias_name) = alias_path {
        //return f(alias_path, entry, alias_name, shortpaths)
    //} else {
        //return PathBuf::from(alias_path.unwrap());
    //}

    /*
    let deps = &sp.deps;
    deps.iter().for_each(|dep| {
        // TODO: Wrap in a while loop later to parse additional paths
        let name = &dep.1;
        dbg!(&name);
        if let Some(dep_shortpath) = shortpaths.get(name) {
            let path = dep_shortpath.path.to_owned();

            let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
            entry = PathBuf::from(output);
        }
        //match dep.0 {
            //ShortpathVariant::Independent => {
                //let name = &dep.1;
                //dbg!(&name);
                ////let dep_shortpath = shortpaths.get(name);
                //let dep_shortpath = shortpaths.find_key_for_value(name).unwrap();
                ////let path = dep_shortpath.path.to_owned();

                ////let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
                //let output = expand_path(entry.to_str().unwrap(), dep_shortpath, name);
                //entry = PathBuf::from(output);
            //}
            //ShortpathVariant::Alias       => {
                //let name = &dep.1;
                ////while let Some(dep_shortpath) = shortpaths.get(name) {
                //dbg!(&name);
                //if let Some(dep_shortpath) = shortpaths.get(name) {
                    //let path = dep_shortpath.path.to_owned();

                    //let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
                    //entry = PathBuf::from(output);
                //}
            //}
            //ShortpathVariant::Environment => {}
        //};
    });
    */

    // Attempt to unwrap the last dependency if any
    //let mut dep = get_shortpath_dependency(&);
    //if let Some(lastdep) = dep {
    //if let Some(lastdep) = dep {
    /*
    let mut entry_string = entry.to_str().unwrap().to_string();
    let mut nested_entry = to_str_slice(entry.to_str().unwrap());
    //dbg!(&nested_entry);
    let mut output = String::new();
    let mut components = entry.components().collect_vec();
    let mut finished = false;
    //while let Some(_) = get_shortpath_type()
    let to_string = |comp: &Component| {comp.as_os_str().to_str().unwrap().to_string()};
    let str_join_path = |s1: &str, s2: &str| {
        let (p1,p2) = (PathBuf::from(s1), PathBuf::from(s2));
        p1.join(p2)
    };
    // What do we need in order to complete the path?

    // How do we go from '$a/bbbb' to aaaa/bbbb ?

    // For every component: ($a, bbbb)
    //  If shortpath_type == alias
    //      get expanded = expand_shortpath with alias path
    //      // let expanded_path = PathBuf::from(expanded);
    //      // let joined = 
    //      let output = expanded
    //      Do expansion
    //  If shortpath_type == environment
    //      Do expansion
    //  Else
    //      break;
    //      

    while !finished {
        //for comp in entry.components() {
        //for comp in components.iter().copied() {
        //for comp in components.pop() {
        //for let Some(comp) in components.pop() {
        if let Some(comp) = components.pop() {
            // commponent: aaaa, bbbb, cccc, dddd
            // commponent: $a, $b, $c,
            let component = to_str_slice(comp.as_os_str().to_str().unwrap());
            let shortpath_type = get_shortpath_type(&component);
            
            match shortpath_type {
                Some(shortpath_variant) => {
                    //components.push(comp.clone());
                    //finished = false;
                    match shortpath_variant {
                        ShortpathVariant::Alias       => {
                            // We know: type, keyname
                            let alias_name = to_string(&comp);
                            let name = parse_alias(&alias_name).unwrap();
                            dbg!(&name);
                            let sp = shortpaths.get(&name).unwrap();
                            dbg!(&sp);
                            let path = &sp.path;
                            dbg!(&path);
                            let expanded = expand_path(entry.to_str().unwrap(), &name, path.to_str().unwrap());
                            dbg!(&expanded);
                            //dbg!("{name}, {:?}, {}, {expanded}", sp, path.display());
                            output = expanded;
                        }
                        ShortpathVariant::Independent => {
                            //let path = to_string(&comp);
                            //dbg!(&path);

                            let leftover = to_string(&comp);
                            let path = str_join_path(&output, &leftover);
                            let str_path = path.to_str().unwrap();
                            dbg!(&str_path);

                            //let name = shortpaths.find_key_for_value(&path).unwrap();
                            //let name = shortpaths.find_key_for_value(&path);
                            let name = shortpaths.find_key_for_value(str_path);
                            dbg!(&name);
                            break;
                            if let Some(name) = name {
                                //let expanded = expand_path(entry.to_str().unwrap(), name, &path);
                                let expanded = expand_path(entry.to_str().unwrap(), name, str_path);
                                dbg!(&expanded);
                                output = expanded;
                            } else {
                                break;
                            }
                        }
                        ShortpathVariant::Environment => {}
                    }
                }
                None => {
                    finished = false;
                    break;
                }
            }
        }
    }
    PathBuf::from(output)
    */

    //while let Some(lastdep) = get_shortpath_dependency(&nested_entry) {
        //dbg!(&entry.to_str().unwrap());
        //dbg!(&entry_string);
        ////dbg!(&to_str_slice(entry.to_str().unwrap()));
    ////let lastdep = dep.take().unwrap();
    ////if let Some(lastdep) = dep {
        //match lastdep.0 {
            //ShortpathVariant::Alias       => {
                ////let name = &lastdep.1;
                ////dbg!(&name);
                //dbg!("In alias");
                //let deppath = &lastdep.1;
                //dbg!(&deppath);


                ////if let Some(dep_shortpath) = shortpaths.get(name) {
                ////if let Some(dep_shortpath) = shortpaths.find_key_for_value(deppath) {
                //let p = format!("${}", deppath);
                //if let Some(valid_keyname) = shortpaths.find_key_for_value(p) {
                    ////let path = dep_shortpath.path.to_owned();

                    ////let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
                    ////let output = expand_path(entry.to_str().unwrap(), &valid_keyname, deppath);
                    ////let expanded_path = expand_path(entry.to_str().unwrap(), &valid_keyname, deppath);
                    //let output = expand_path(entry.to_str().unwrap(), &valid_keyname, deppath);
                    //dbg!(&output);
                    //entry_string = output.clone();
                    //nested_entry = to_str_slice(output.clone());
                    ////entry_string = nested_entry.iter().collect();
                    ////entry = PathBuf::from(output);
                //}
                //[>
                //let name = lastdep.1;
                //dbg!(&name);
                ////if let Some(valid_shortpath) = shortpaths.get(&name) {
                //if let Some(valid_name) = shortpaths.find_key_for_value(&name) {
                    ////let path = dep_shortpath.path.to_owned();
                    ////let path = valid_shortpath.path.to_owned();
                    //let path = name;
                    //let output = expand_path(entry.to_str().unwrap(), valid_name, &path);

                    //// Set the new nested_entry
                    //nested_entry = to_str_slice(output.clone());
                    //entry = PathBuf::from(output);
                    ////let path = dep_shortpath.path.to_owned();
                    ////let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
                    ////entry = PathBuf::from(output);
                //}
                //*/
            //}
            //ShortpathVariant::Independent => {
                //dbg!("In independent");
                //let deppath = &lastdep.1;
                //dbg!(&deppath);


                ////if let Some(dep_shortpath) = shortpaths.get(name) {
                ////if let Some(dep_shortpath) = shortpaths.find_key_for_value(deppath) {
                //let p = format!("${}", deppath);
                ////if let Some(valid_keyname) = shortpaths.find_key_for_value(deppath) {
                //if let Some(valid_keyname) = shortpaths.find_key_for_value(p) {
                    ////let path = dep_shortpath.path.to_owned();

                    ////let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
                    //let output = expand_path(entry.to_str().unwrap(), &valid_keyname, deppath);
                    //entry_string = output.clone();
                    //nested_entry = to_str_slice(output.clone());
                    ////entry = PathBuf::from(output);
                //}
                ////let name = &lastdep.1;
                ////dbg!(&name);
                ////if let Some(dep_shortpath) = shortpaths.get(name) {
                    ////let path = dep_shortpath.path.to_owned();

                    ////let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
                    ////entry = PathBuf::from(output.clone());
                    ////nested_entry = to_str_slice(output);
                ////}
                //[>
                //let name = lastdep.1;
                //dbg!(&name);
                ////if let Some(valid_shortpath) = shortpaths.get(&name) {
                //if let Some(valid_name) = shortpaths.find_key_for_value(&name) {
                    ////let path = dep_shortpath.path.to_owned();
                    ////let path = valid_shortpath.path.to_owned();
                    //let path = name;
                    //let output = expand_path(entry.to_str().unwrap(), valid_name, &path);

                    //// Set the new nested_entry
                    //nested_entry = to_str_slice(output.clone());
                    //entry = PathBuf::from(output);
                    ////let path = dep_shortpath.path.to_owned();
                    ////let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
                    ////entry = PathBuf::from(output);
                    //break;
                //}
                //*/
            //}
            //ShortpathVariant::Environment => {
                //break;
            //}
        //};
        //break;

        /*
        let name = lastdep.1;
        dbg!(&name);
        //if let Some(valid_shortpath) = shortpaths.get(&name) {
        if let Some(valid_name) = shortpaths.find_key_for_value(&name) {
            //let path = dep_shortpath.path.to_owned();
            //let path = valid_shortpath.path.to_owned();
            let path = name;
            let output = expand_path(entry.to_str().unwrap(), valid_name, &path);

            // Set the new nested_entry
            nested_entry = to_str_slice(output.clone());
            entry = PathBuf::from(output);
            //let path = dep_shortpath.path.to_owned();
            //let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
            //entry = PathBuf::from(output);
        */

        //} else {
            //break;
        //}
        
        //if let Some(dep_shortpath) = shortpaths.get(name) {
            //let path = dep_shortpath.path.to_owned();

            //let output = expand_path(entry.to_str().unwrap(), name, path.to_str().unwrap());
            //entry = PathBuf::from(output);
        //}
    //}
    //entry
}

// Commands
pub fn add_shortpath(shortpaths: &mut SP, name: String, path: PathBuf) {
    let shortpath = Shortpath::new(path, None, vec![]);
    shortpaths.insert(name, shortpath);
}

pub fn remove_shortpath(shortpaths: &mut SP, current_name: &str) -> Option<Shortpath> {
    shortpaths.remove(current_name)
}

pub fn find_unreachable(shortpaths: &SP) -> IndexMap<&String, &Shortpath> {
    let unreachable: IndexMap<&String, &Shortpath> = shortpaths.iter()
        .filter(|(_, path)| { !path.path.exists() || path.path.to_str().is_none() }).collect();
    unreachable
}

/// List any broken or unreachable paths
pub fn check_shortpaths(shortpaths: &mut SP) {
    let unreachable = find_unreachable(shortpaths);
    unreachable.iter().for_each(|(alias_name, alias_path)|
        println!("{} shortpath is unreachable: {}", alias_name, alias_path.path.display()));
    println!("Check Complete");
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
    let automode_fn = |shortpaths: &SP, sp: &Shortpath, results: Vec<DirEntry>| {
        let first = results.first().unwrap();
        let path = first.path().to_owned();
        let name = shortpaths.find_key_for_value(path.to_str().unwrap()).unwrap();
        (name.to_owned(), path)
    };

    // Manual: Provide options at runtime for the user
    let manualmode_fn = |shortpaths: &SP, sp: &Shortpath, results: Vec<DirEntry>| {
        let path = sp.path.to_owned();
        let name = shortpaths.find_key_for_value(path.to_str().unwrap()).unwrap();
        //let (name, path) = (sp.name.to_owned(), sp.path.to_owned());
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
            let current_path = sp.path.to_owned();
            let (name, path) = resolve_fn(shortpaths, sp, results);

            if path != current_path {
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
pub fn export_shortpaths(shortpaths: &SP, export_type: &str, output_file: Option<&String>) -> PathBuf {
    let exp = get_exporter(export_type)
        .set_shortpaths(shortpaths);
    let dest = exp.prepare_directory(output_file);
    exp.write_completions(dest)
}

/** Update a single shortpath's alias name or path
  * Changes the name or path if given and are unique */
pub fn update_shortpath(shortpaths: &mut SP, current_name: &str, name: Option<&String>, path: Option<&String>) {
    let entry_exists = || { shortpaths.get(current_name).is_some() }; 

    let update_path = |new_path: String, shortpaths: &mut SP| {
        let shortpath = Shortpath::new(PathBuf::from(new_path), None, vec![]);
        shortpaths.insert(current_name.to_owned(), shortpath);
    };
    let update_name = |new_name: String, shortpaths: &mut SP| {
        let path = shortpaths.remove(current_name).unwrap();
        shortpaths.insert(new_name, path);
    };

    match (entry_exists(), name, path) {
        (true, Some(new_name), _) => { update_name(new_name.to_owned(), shortpaths); }
        (true, _, Some(new_path)) => { update_path(new_path.to_owned(), shortpaths); }
        (_, _, _)              => { println!("Nothing to do");}
    }
}
