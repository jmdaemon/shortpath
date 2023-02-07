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

- **Better Redundancy:** If a directory is moved, the shortpath is updated, and every application that uses the shortpath functions as intended.
- **Environment Variable Support:** Make use of environment variables as path names using the `${env:my_env_var}` syntax.
- **Nested Definitions:** Embed one shortpath inside of another with the `$alias_path` syntax.
- **Shell Completions:** Shortpaths can export shell completions for paths. Supported shells are: bash, powershell.
    - **(Not Yet Implemented)**
- **Easy Alias Path Management:** Adding new shortpaths is as easy as `shortpath add [name] [path]`
- **Centralization:** One configuration available for use in many applications.
- **Slightly Better Security:** The permissions set for your shortpath config is editable only by the current user.
    The shell completions file is read + user executable only.
    - **(Not Yet Implemented)**

## Problem

When an application relies on a directory path, the moment that directory is moved elsewhere,
the path breaks the application functionality.With `shortpaths` its possible for your
path to still be usable even if the directory was moved elsewhere.

## How It Works

Shortpath's key feature is being able to nest themselves as aliases inside other shortpaths.
This allows you to expand the shortpath at runtime, leading to a more resilient directory link
at the expense of the cost to expand.

Combined with the shell update hooks, its possible for the user to define a path and seamlessly
work without having to worry about updating aliases again. **(Not Yet Implemented)**

> What happens if I one of my paths is still broken?

If the path is breakable, then you can make use of `shortpaths` additional feature:
- `shortpath resolve`: Lets you manually/automatically find and fix unreachable or broken shortpaths. **(Partially Implemented)**

If you would prefer to work through your paths manually then `shortpaths` has you covered:
- `shortpath list`: Displays your current configuration
- `shortpath list [alias]`: Displays the path of the corresponding `shortpath`.

See [below](#usage) for usage on removing shortpaths.

## Usage

```bash
shortpath add "name" "path"

shortpath remove -n "name" # Remove by name
shortpath remove -p "path" # Remove by path

# Warns the user for unreachable paths
shortpath check

# Resolve broken shortpath links if any
shortpath resolve

# Update
shortpath update "current_name" -n "new_name" # Renames shortpath
shortpath update "current_name" -p "new_path" # Change shortpath directory

# Exports shell completions
shortpath export bash       # Bash completions
shortpath export powershell # Powershell completions
```

## Shell Completions

**Not Yet Implemented**

If you want shortpaths to automatically update your shortpaths config when
you're working with files and/or folders in the shell, then load the
helper scripts in `hooks`.

### Bash

**Not Yet Implemented**

Source `hooks/shortpaths_hooks.sh` in your `.bashrc`. Or if you'd prefer to generate these manually:

```bash
shortpaths export bash
```

### Powershell
**Not Yet Implemented**

Source `hooks/shortpaths_hooks.ps1` in your `$profile`. Or if you'd prefer to generate these manually:

```bash
shortpaths export powershell
```

## Issues

### API

The API is very messy in some areas:
- `populate_dependencies`, `populate_expanded_paths`,
    `Shortpaths::default()`, `Shortpaths::to_disk()` are very messy
    and also use some unnecessary `filter_map()`s, as linted by clippy.
    For more on this issue see below:
- Not enough unit tests for shortpaths + difficulty in writing out tests.
    There's too much boilerplate in initializing and preparing the shortpaths.
- `expand_shortpath`, `fold_shortpath` are not fallible. This is a bug however since
    if the input is not formatted properly, then we should definitely not use it.
- Shortpaths is probably not as lightweight as it could be with the various clones used,
    as a workaround for the borrow checker. Once the shortpath config file is read,
    there should be no need to clone the data at all.
- The `ShortpathsBuilder` currently doesn't support various other features outlined
    in the `arch.rs` architecture wishlist.
- There are no `powershell` completions yet.
- There is little documentation for how to use `shortpaths` as a library in
    other applications.

The current api for both creating and serializing shortpaths is duplicated
across various files in shortpaths, namely `app.rs`, `bash.rs`, `shortpaths.rs`. A
single `api.rs` (name in progress) that defines the same functionality and types
used across all these files is to be preferred.

## Binary

The binary is still missing a few key features:
- Ability to prompt users for a path.
- For the command line interface, prefer using clap-derive over clap-builder for
    better reuseability and composibility.
- Create an optional `-y` parameter.
- `shortpath remove --paths [shortpath1] [shortpath2]` command that can accept a vector of paths.
    - Use the `-y` parameter to skip prompting, and move on to deletion.
- `shortpath list` and `shortpath lists`
    - `shortpath list [shortpath]`
        - Prints out the directory the Shortpath points to
    - `shortpath list`
        - Prints out all shortpaths
- `shortpath refresh`: Platform specific.
    Unsets all shortpath variables for the platform (bash, powershell), sets them again, and then refreshes the current shell
    with the new definitions.
- `shortpath resolve [resolve_type] [options]`: Needs to have a few more additional options, and be written
    to be more modular. This function will be featureful and needs to play loose to support lots of options.
- `shortpath update_hook [args]`
    or `shortpath hook update [args]`
    or `shortpath update_hook [src] [dest]`:
    Check if the shortpath exists in our config, and runs the command to update and save our updated path.
- `shortpath remove_hook [args]`
    or `shortpath hook remove [args]`
    or `shortpath remove_hook --paths [shortpaths]`:
    Check if the shortpath exists in our config, and runs the command to remove the path.
- Note for the hooks:
    - If parsing is done in the binary then clap argument parsing must be disabled for these hooks.
        - This is because if there are binary specific arguments given, like `mv src dest -p`,
            then the flags will break clap parsing when it detects that those options are missing
    - If parsing is not done in the binary, and the script hooks are successful, then
        we can make use of clap argument parsing.
    - Alternative Solutions:
        - Since you cannot alias functions in `powershell`,
            shortpaths could provide `mv` and/or `rm` commands that implement similar bare minimum
            functionality that wrap the native platform commands instead of providing an update hook.
            These would be like: `sp mv`, `sp rm`.
    - The hooks would make use of `FindKeyIndexMapExt` to get the value from the key provided.
        **NOTE** Be sure to attempt it with the path itself, and the path expanded (if its an alias).

- Detailed manpage file for Linux users.

## TODO

- Profile/benchmark shortpaths
    - Benches crate?

- Consider `tracing` crate.
