
#[test]
fn test_serialize_bash() {
    use shortpaths::{
        shortpaths::{SPD, Shortpath, ShortpathsBuilder},
        export::{Export, bash::BashExporter},
    };

    use std::path::PathBuf;

    use log::LevelFilter;
    use pretty_env_logger::formatted_timed_builder;

    // Enable debug statements
    formatted_timed_builder().filter_level(LevelFilter::Trace).init();

    // Init
    let sp_paths = vec![
        Shortpath::new("d".to_owned(), PathBuf::from("$a/dddd"), None, vec![]),
        Shortpath::new("c".to_owned(), PathBuf::from("$b/cccc"), None, vec![]),
        Shortpath::new("b".to_owned(), PathBuf::from("$a/bbbb"), None, vec![]),
        Shortpath::new("a".to_owned(), PathBuf::from("aaaa"), None, vec![]),
    ];
    let mut sp_builder = ShortpathsBuilder::new(sp_paths);

    let shortpaths = sp_builder.build().unwrap();

    let mut exp = BashExporter::default();
    exp.set_shortpaths(&shortpaths);

    // Test
    let actual = exp.gen_completions();
    let expect = "#!/bin/bash\n\nexport a=\"aaaa\"\nexport b=\"$a/bbbb\"\nnexport c=\"$b/cccc\nexport d=\"$a/dddd\"\n";
    assert_eq!(actual, expect, "Bash shell completions are generated in the correct order");
}
