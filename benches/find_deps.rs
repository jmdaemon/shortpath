use shortpaths::{
    shortpaths::{Shortpath, ShortpathsBuilder},
    export::{Export, bash::BashExporter},
};

use std::path::PathBuf;

use indexmap::indexmap;
use log::LevelFilter;
use pretty_env_logger::formatted_timed_builder;

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_nested_serialize_bash() {
    // Init
    let sp_paths = indexmap!{
        "d".to_owned() => Shortpath::new(PathBuf::from("$c/dddd"), None, vec![]),
        "c".to_owned() => Shortpath::new(PathBuf::from("$b/cccc"), None, vec![]),
        "b".to_owned() => Shortpath::new(PathBuf::from("$a/bbbb"), None, vec![]),
        "a".to_owned() => Shortpath::new(PathBuf::from("aaaa"), None, vec![]),
    };
    let mut sp_builder = ShortpathsBuilder::new(sp_paths);

    let shortpaths = sp_builder.build().unwrap();

    //let mut exp = BashExporter::default();
    //exp.set_shortpaths(&shortpaths);
    let exp = BashExporter::default()
        .set_shortpaths(&shortpaths);
    //let exp = get_exporter("bash")

    // Test
    let actual = exp.gen_completions();
    let expect = "#!/bin/bash\n\nexport a=\"aaaa\"\nexport b=\"$a/bbbb\"\nexport c=\"$b/cccc\"\nexport d=\"$c/dddd\"\n";
    assert_eq!(actual, expect, "Bash shell completions are generated in the correct order");
    assert_eq!(1, 0);
}

fn criterion_benchmark(c: &mut Criterion) {
    // Enable debug statements
    formatted_timed_builder().filter_level(LevelFilter::Trace).init();

    c.bench_function("bench_nested_serialize_bash", |b| b.iter(|| bench_nested_serialize_bash));
    //c.bench_function("bench_nested_serialize_bash", |b| b.iter(|| bench_nested_serialize_bash(black_box())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
