use shortpaths::shortpaths::{
    FindKeyIndexMapExt,
    sort_shortpaths,
};
use crate::helpers::{
    shortpaths_default,
    enable_logging,
    setup_shortpaths,
};

#[test]
fn set_logger() {
    enable_logging();
}

// Test the shortpath library internals

#[test]
fn test_shortpaths() {
    let sp_im = setup_shortpaths(shortpaths_default);
    sp_im.iter().for_each(|p| println!("{:?}", p));

    // Test find_key
    let key = sp_im.find_key_for_value("$a/bbbb");
    println!("{:?}", key);
    assert_ne!(None, key, "Can find keys from &str values");

    let key = sp_im.find_key_for_value("$a/bbbb".to_string());
    println!("{:?}", key);
    assert_ne!(None, key, "Can find keys from String values");

    // Test sort_shortpaths
    println!("Sorted list of shortpaths");
    let sorted = sort_shortpaths(sp_im);
    sorted.iter().for_each(|p| println!("{:?}", p));
}
