use crate::helpers::{
    shortpaths_default,
    shortpaths_nested,
    setup_shortpaths,
};

use shortpaths::export::{Export, bash::BashExporter};

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
