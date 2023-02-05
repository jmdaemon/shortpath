#[test]
fn test_serialize_bash() {
    use shortpaths::shortpaths::{Shortpath, SPT, ShortpathsBuilder};
    use shortpaths::export::bash::BashExporter;
    use shortpaths::export::Export;
    use std::path::PathBuf;

    use log::LevelFilter;
    use pretty_env_logger::formatted_timed_builder;

    // Enable debug statements
    formatted_timed_builder().filter_level(LevelFilter::Trace).init();

    // Init
    let sp_paths = vec![
        Shortpath::new(SPT::new_path("d", PathBuf::from("$a/dddd")), None, None),
        Shortpath::new(SPT::new_path("c", PathBuf::from("$b/cccc")), None, None),
        Shortpath::new(SPT::new_path("b", PathBuf::from("$a/bbbb")), None, None),
        Shortpath::new(SPT::new_path("a", PathBuf::from("aaaa")), None, None),
    ];
    let mut sp_builder = ShortpathsBuilder::new(sp_paths);

    let shortpaths = sp_builder.build().unwrap();

    let mut exp = BashExporter::default();
    exp.set_shortpaths(&shortpaths);

    // Test
    let actual = exp.gen_completions(None);
    let expect = "#!/bin/bash\n\nexport a=\"aaaa\"\nexport b=\"$a/bbbb\"\nexport d=\"$a/dddd\"\nexport c=\"$b/cccc\"\n";
    assert_eq!(actual, expect);
}
