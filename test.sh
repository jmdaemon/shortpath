#!/bin/bash

# We have to test using this script because the operation for find_deps is somewhat long, and it is not possible to
# test the two in parallel and obtain correctly formatted output
cargo test -- --test-threads=1
