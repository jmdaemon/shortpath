#[allow(unused_imports)]
use crate::helpers::{enable_logging, enable_logging_single_test};
use crate::helpers::{
    shortpaths_default,
    setup_shortpaths,
};
use shortpaths::{
    app::{ResolveType, Mode},
    builder::ShortpathsBuilder,
    shortpaths::{
        FindKeyIndexMapExt,
        Shortpath, resolve,
    },
};

use std::path::PathBuf;

use indexmap::indexmap;

// Test the shortpath library internals

#[test]
fn test_shortpaths_find_key() {
    enable_logging();
    let sp_im = setup_shortpaths(shortpaths_default);
    sp_im.iter().for_each(|p| println!("{:?}", p));

    let key = sp_im.find_key_for_value("$a/bbbb");
    println!("{:?}", key);
    assert_ne!(None, key, "Can find keys from &str values");

    let key = sp_im.find_key_for_value("$a/bbbb".to_string());
    println!("{:?}", key);
    assert_ne!(None, key, "Can find keys from String values");
}

#[test]
fn test_shortpaths_resolve() {
    enable_logging();
    //enable_logging_single_test();
    let unreachable = indexmap! {
        "DNE".to_owned() => Shortpath::new(PathBuf::from("~/Workspace/test/DNE"), None),
    };
    
    let builder = ShortpathsBuilder::from(unreachable);
    let paths = builder.build().unwrap();
    let mut shortpaths = paths.shortpaths;

    // Construct function parameters
    let resolve_type = ResolveType::Matching;
    let mode = Mode::Automatic;
    let dry_run = true;
    
    resolve(&mut shortpaths, resolve_type, mode, dry_run);
    //assert_eq!(1, 0, "Force show debug statements");
    //assert_eq!(1, 2, "builder does not construct objects that don't work");
}
