use shortpaths::{
    shortpaths::{Shortpath, SP},
    builder::{ShortpathsBuilder, ShortpathOperationsExt},
    export::{Export, bash::BashExporter},
};

use std::path::PathBuf;

use indexmap::indexmap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Benchmarks

fn bench_populdate_expanded_paths(shortpaths: &SP) -> SP {
    shortpaths.populate_expanded_paths()
}

fn bench_nested_serialize_bash(shortpaths: &SP) -> String {
    let exp = BashExporter::default()
        .set_shortpaths(shortpaths);
    exp.gen_completions()
}

fn criterion_benchmark(c: &mut Criterion) {
    // Enable debug statements
    //use log::LevelFilter;
    //use pretty_env_logger::formatted_timed_builder;
    //formatted_timed_builder().filter_level(LevelFilter::Trace).init();

    // Initialization
    let sp_paths = indexmap!{
        "d".to_owned() => Shortpath::new(PathBuf::from("$c/dddd"), None),
        "c".to_owned() => Shortpath::new(PathBuf::from("$b/cccc"), None),
        "b".to_owned() => Shortpath::new(PathBuf::from("$a/bbbb"), None),
        "a".to_owned() => Shortpath::new(PathBuf::from("aaaa"), None),
    };

    let paths = ShortpathsBuilder::from(sp_paths).build().unwrap();
    let shortpaths = paths.shortpaths;

    // Benchmark
    c.bench_function("bench_populdate_expanded_paths sp_paths",
        |b| b.iter(|| bench_populdate_expanded_paths(black_box(&shortpaths))));

    c.bench_function("bench_nested_serialize_bash",
        |b| b.iter(|| bench_nested_serialize_bash(black_box(&shortpaths))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
