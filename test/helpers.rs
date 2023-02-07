use shortpaths::shortpaths::{Shortpath, ShortpathsBuilder, SP};

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
    // Enable debug statements
    //formatted_timed_builder().filter_level(LevelFilter::Trace).init();
}

/// Initialize shortpaths
pub fn setup_shortpaths(get_shortpaths: impl Fn() -> SP) -> SP {
    let sp_paths = get_shortpaths();
    let mut sp_builder = ShortpathsBuilder::new(sp_paths);
    sp_builder.build().unwrap()
}
