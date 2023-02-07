use shortpaths::{
    shortpaths::{Shortpath, ShortpathsBuilder, SP},
    export::{Export, bash::BashExporter},
};

use std::path::PathBuf;

use indexmap::indexmap;
use log::LevelFilter;
use pretty_env_logger::formatted_timed_builder;

// Fixtures

// Different shortpath configurations to choose from
fn shortpaths_default() -> SP {
    let sp_paths = indexmap!{
        "d".to_owned() => Shortpath::new(PathBuf::from("$a/dddd"), None, vec![]),
        "c".to_owned() => Shortpath::new(PathBuf::from("$b/cccc"), None, vec![]),
        "b".to_owned() => Shortpath::new(PathBuf::from("$a/bbbb"), None, vec![]),
        "a".to_owned() => Shortpath::new(PathBuf::from("aaaa"), None, vec![]),
    };
    sp_paths
}

fn shortpaths_nested() -> SP {
    let sp_paths = indexmap!{
        "d".to_owned() => Shortpath::new(PathBuf::from("$c/dddd"), None, vec![]),
        "c".to_owned() => Shortpath::new(PathBuf::from("$b/cccc"), None, vec![]),
        "b".to_owned() => Shortpath::new(PathBuf::from("$a/bbbb"), None, vec![]),
        "a".to_owned() => Shortpath::new(PathBuf::from("aaaa"), None, vec![]),
    };
    sp_paths
}

/// Initialize shortpaths
fn setup_shortpaths(get_shortpaths: impl Fn() -> SP) -> SP {
    let sp_paths = get_shortpaths();
    let mut sp_builder = ShortpathsBuilder::new(sp_paths);
    sp_builder.build().unwrap()
}

/// This ensures that we always set the logger
#[test]
fn enable_logging() {
    // Enable debug statements
    formatted_timed_builder().filter_level(LevelFilter::Trace).init();
}

#[test]
fn test_serialize_bash() {
    let shortpaths = setup_shortpaths(shortpaths_default);

    let exp = BashExporter::default()
        .set_shortpaths(&shortpaths);

    // Test
    let actual = exp.gen_completions();
    let expect = "#!/bin/bash\n\nexport a=\"aaaa\"\nexport b=\"$a/bbbb\"\nexport c=\"$b/cccc\"\nexport d=\"$a/dddd\"\n";
    assert_eq!(actual, expect, "Bash shell completions are generated in the correct order");
}

#[test]
fn test_nested_serialize_bash() {
    let shortpaths = setup_shortpaths(shortpaths_nested);
    let exp = BashExporter::default()
        .set_shortpaths(&shortpaths);

    // Test
    let actual = exp.gen_completions();
    let expect = "#!/bin/bash\n\nexport a=\"aaaa\"\nexport b=\"$a/bbbb\"\nexport c=\"$b/cccc\"\nexport d=\"$c/dddd\"\n";
    assert_eq!(actual, expect, "Bash shell completions are generated in the correct order");
    assert_eq!(1, 0);
}
