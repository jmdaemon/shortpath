#!/bin/bash

# Prepares benchmarking symlink
ln -s ../benches/criterion target/

# Options to always run when benchmarking
cargo bench -- --verbose
