use shortpaths::{
    shortpaths::{Shortpath, SP},
    builder:: ShortpathsBuilder,
};

use std::path::PathBuf;

use indexmap::indexmap;
use log::LevelFilter;
use pretty_env_logger::formatted_timed_builder;

// Different shortpath configurations to choose from
pub fn shortpaths_default() -> SP {
    let sp_paths = indexmap!{
        "d".to_owned() => Shortpath::new(PathBuf::from("$a/dddd"), None),
        "c".to_owned() => Shortpath::new(PathBuf::from("$b/cccc"), None),
        "b".to_owned() => Shortpath::new(PathBuf::from("$a/bbbb"), None),
        "a".to_owned() => Shortpath::new(PathBuf::from("aaaa"), None),
    };
    sp_paths
}

pub fn shortpaths_nested() -> SP {
    let sp_paths = indexmap!{
        "d".to_owned() => Shortpath::new(PathBuf::from("$c/dddd"), None),
        "c".to_owned() => Shortpath::new(PathBuf::from("$b/cccc"), None),
        "b".to_owned() => Shortpath::new(PathBuf::from("$a/bbbb"), None),
        "a".to_owned() => Shortpath::new(PathBuf::from("aaaa"), None),
    };
    sp_paths
}

/// This ensures that we always set the logger
pub fn enable_logging() {
    // Enable log statements
    match formatted_timed_builder().filter_level(LevelFilter::Trace).is_test(true).try_init() {
        Ok(_) => { },
        Err(_) => {
            println!("Logger already initialized, skipping initialization.")
        }
    };
}

/// Enables all log statements (with color) for a single test only
pub fn enable_logging_single_test() {
    formatted_timed_builder().filter_level(LevelFilter::Trace).init();
}

/// Initialize shortpaths
pub fn setup_shortpaths(get_shortpaths: impl Fn() -> SP) -> SP {
    let sp_paths = get_shortpaths();
    let paths =
        ShortpathsBuilder::from(sp_paths).build().unwrap();
    paths .shortpaths
}
