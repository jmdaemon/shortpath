use shortpaths::{
    shortpaths::{Shortpath, ShortpathsBuilder, SP, populate_expanded_paths},
    export::{Export, bash::BashExporter},
};

use std::path::PathBuf;

use indexmap::indexmap;
use log::LevelFilter;
use pretty_env_logger::formatted_timed_builder;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Benchmarks
fn bench_populate_dependencies(mut shortpaths: SP) -> SP {
    populate_expanded_paths(&mut shortpaths)
}

fn bench_nested_serialize_bash(shortpaths: SP) {
    let exp = BashExporter::default()
        .set_shortpaths(&shortpaths);

    // Test
    let actual = exp.gen_completions();
    let expect = "#!/bin/bash\n\nexport a=\"aaaa\"\nexport b=\"$a/bbbb\"\nexport c=\"$b/cccc\"\nexport d=\"$c/dddd\"\n";
    assert_eq!(actual, expect, "Bash shell completions are generated in the correct order");
    assert_eq!(1, 0);
}

fn criterion_benchmark(c: &mut Criterion) {
    // Enable debug statements
    formatted_timed_builder().filter_level(LevelFilter::Trace).init();

    // Initialization
    let sp_paths = indexmap!{
        "d".to_owned() => Shortpath::new(PathBuf::from("$c/dddd"), None),
        "c".to_owned() => Shortpath::new(PathBuf::from("$b/cccc"), None),
        "b".to_owned() => Shortpath::new(PathBuf::from("$a/bbbb"), None),
        "a".to_owned() => Shortpath::new(PathBuf::from("aaaa"), None),
    };

    let mut sp_builder = ShortpathsBuilder::new(sp_paths);
    let shortpaths = sp_builder.build().unwrap();

    c.bench_function("bench_populate_dependencies sp_paths",
        |b| b.iter(|| bench_populate_dependencies(black_box(shortpaths.clone()))));

    //c.bench_function("bench_nested_serialize_bash", |b| b.iter(|| bench_nested_serialize_bash(shortpaths.clone())));

    //c.bench_function("bench_nested_serialize_bash", |b| b.iter(|| bench_nested_serialize_bash(black_box())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
