# Shortpaths

Shortpaths is a Rust program for providing better path aliases to files or directories.

## Table of Contents
1. [Features](#features)
2. [Problem](#problem)
3. [How It Works](#how-it-works)
4. [Usage](#usage)
5. [Shell Completions](#shell-completions)
    - [Bash](#bash)
    - [Powershell](#powershell)
6. [Issues](#issues)

## Features

- Better Redundancy: If a directory is moved, the shortpath is updated, and every application that uses the shortpath functions as intended.
- Environment Variable Support: Make use of environment variables as path names using the `${env:my_env_var}` syntax.
- Nested Definitions: Embed one shortpath inside of another with the `$alias_path` syntax.
- Shell Completions: Shortpaths can export shell completions for paths. Supported shells are: bash, powershell (Not Yet Implemented).
- Easy Alias Path Management: Adding new shortpaths is as easy as `shortpath add [name] [path]`
- Centralization: One configuration available for use in many applications.
- Slightly Better Security: The permissions set for your shortpath config is editable only (Not Yet Implemented) by the current user.
    The shell completions file is read + user executable only (Not Yet Implemented).

## Problem

When an application relies on a directory path, the moment that directory is moved elsewhere,
the path breaks the application functionality.With `shortpaths` its possible for your
path to still be usable even if the directory was moved elsewhere.

## How It Works

Shortpath's key feature is being able to nest themselves as aliases inside other shortpaths.
This allows you to expand the shortpath at runtime, leading to a more resilient directory link
at the expense of the cost to expand.

Combined with the shell update hooks, its possible for the user to define a path and seamlessly
work without having to worry about updating aliases again.

> What happens if I one of my paths is still broken?

If the path is breakable, then you can make use of `shortpaths` additional feature:
- `shortpath resolve`: Lets you manually/automatically find and fix unreachable or broken shortpaths.

If you would prefer to work through your paths manually then `shortpaths` has you covered:
- `shortpath list`: Displays your current configuration
- `shortpath list [alias]`: Displays the path of the corresponding `shortpath`.

See [below](#usage) for usage on removing shortpaths.

## Usage

```bash
# Add a new shortpath
shortpath add "name" "path"

# Remove a shortpath
shortpath remove -n "name" # Remove by name
shortpath remove -p "path" # Remove by path

# Warns the user for unreachable paths
shortpath check

# Resolve broken shortpath links if any
shortpath resolve

# Update a shortpath
shortpath update "current_name" -n "new_name" # Renames shortpath
shortpath update "current_name" -p "new_path" # Change shortpath directory

# Exports shell completions
shortpath export bash       # Bash completions
shortpath export powershell # Powershell completions
```

## Shell Completions

### Bash

If you'd like to get resilient shortpath completions for bash, add the following to your `.bashrc`:

```bash
mv() {
    # TODO
}

rm() {
    # TODO
}
```

Generate and source the shell completions with:

```bash
# TODO: Make install script to install bash/powershell completions
cargo b
mv target/completions/shortpath.bash /usr/share/bash-completion/completions
```

### Powershell

To get resilient completions for powershell, add the following to your `$profile`:

```ps1
move() {
    # TODO
}

remove() {
    # TODO
}
```

Generate and source the shell completions with:

```bash
cargo b
mv target/completions/shortpath.ps1 $profile/shortpath.ps1
```

## Issues

The current api for both creating and serializing shortpaths is duplicated
across various files in shortpaths, namely `app.rs`, `bash.rs`, `shortpaths.rs`.

A single `api.rs` (name in progress) that defines the same functionality used across
all these files is to be preferred.

When shortpaths expands or folds a path, the function should return a result, in case the path wasn't correctly formatted.

The api should be modified to make it easier and faster to load shortpath configs from disk.

## TODO

- Profile/benchmark shortpaths
    - Benches crate?
- Write more unit tests
- Generate man pages for shortpaths (cli)
- Export powershell completions
- Create and Complete Documentation

- Consider `tracing` crate.
