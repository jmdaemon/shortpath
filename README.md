# Shortpaths

Shortpaths is a Rust program for providing better path aliases to files or directories.

## Table of Contents
1. [Problem](#problem)
2. [How It Works](#how-it-works)
3. [Usage](#usage)
4. [Features](#features)
5. [Shell Completions](#shell-completions)
    - [Bash](#bash)
    - [Powershell](#powershell)
6. [Issues](#issues)

## Problem

When an application relies on a directory path, the moment that directory is moved elsewhere,
the path breaks the application functionality. With `shortpaths` its possible for your
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

See [here](#usage) for more information on managing shortpaths.

### Why not use Environment Variables?

Solutions that would involve the use of environment variables would be buggy,
ad-hoc, non portable and limited only to specific use cases.

By creating and using shortpaths, we can have the same functionality as
a environment variable paths, while being portable to other platforms
and reusable by other applications.

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

## Features

- **Better Redundancy:** If a directory is moved, the shortpath is updated, and every application that uses the shortpath functions as intended.
- **Environment Variable Support:** Make use of environment variables as path names using the `${env:my_env_var}` syntax.
- **Nested Definitions:** Embed one shortpath inside of another with the `$alias_path` syntax.
- **Shell Completions:** Shortpaths can export shell completions for paths. Supported shells are: bash, powershell.
- **Easy Alias Path Management:** Adding new shortpaths is as easy as `shortpath add [name] [path]`
- **Centralization:** One configuration available for use in many applications.
- **Slightly Better Security:** Exported variable configs are `rwx` only by the current user and readonly for everyone else.

## Shell Completions

If you want shortpaths to automatically update your shortpaths config when
you're working with files and/or folders in the shell, then load the
helper scripts in `hooks`.

### Bash

Source `hooks/shortpaths_hooks.sh` in your `.bashrc` or add its contents to your `.bashrc`.
These hooks wrap the `mv` and `rm` utilities to call `shortpaths hook {move/remove}` respectively.
The hook checks if a given path is also a shortpath, and attempts to update or remove it.

### Powershell

**Not Yet Implemented**

Source `hooks/shortpaths_hooks.ps1` in your `$profile`. Or if you'd prefer to generate these manually:

```bash
shortpaths export powershell
```

## Issues

- Environment variables will expand awkwardly when not used in strict mode.
    In order to make environment variables less strict, there needs to be
    better filtering. However the current implementation is alright for now.

### API

Currently, folding and expanding shortpaths is messy and ad-hoc, and filled with many side-effects.
The API should be refactored to have a unified and logical method for folding and expanding shortpaths.
The API must

- Expand/Fold a single shortpaths
- Expand/Fold multiple shortpaths
- Expand/Fold nested shortpath aliases
- Expand/Fold nested shortpath environment variables (strictly)

As a library, there needs to be a way to manage multiple shortpath configs.
Applications that make use of the library will need to create their own shortpath configs
and the shortpath binary needs to make use of these configurations in order to provide
better application support.

#### Folding
- Eliminate as many duplicate and overlapping paths as possible.
    One big issue currently is that if there are many shortpaths with overlapping paths,
    there will be more breakages than if they relied on their respective GCD paths.

- There needs to be a way to exhaustively determine shortest paths given the current aliases such that:
    - Shortpath aliases have higher priority than environment variables
    - All shortpath aliases are folded exhaustively

#### Library

- Shortpaths config isn't sanitized before it is used in `expand_shortpath` and `fold_shortpath`.
    - The config file strings aren't checked for correctness before they are used,
        thereby allowing for more undefined behavior at runtime.

- Leverage rustdoc to document library, and provide examples in `examples`
- Write more unit tests.

Things that are nice to have but are not necessary:

- Caching/Reusing `full_paths` from previous paths given they
    contain matching aliases, could lead to a potential order of magnitude speedup.
- Enable feature to expand and fold environment variables.
- Use custom GAT iterator to simplify expand_shortpath

- Create feature to detect shortpaths that don't exist on disk

- Parsing of the shortpath aliases is very hacky. One solution is to upgrade to a proper Lexer Token Parser.
    This new parser would be required to:
    - Iterate through nested shortpaths
    Additional, this parser could make it easier to:
    - Detect shortpaths that don't exist
        - In this case, shortpath could continue to the next path on erroring out.
    - Incorporate more complex shortpath replacements
    - Be more efficient & use less memory on the stack.
    - Incorporate nesting limit sizes `alias_nest_limit=3` as a config option.
    - Offer easier means of exposing & exporting the shortpaths to disk

- Add custom option configurations. Some ideas for config options:
    - `alias_nest_limit`: Set nesting limit for parsing shortpaths
    - `strict`: Error and terminate immediately on the first invalid shortpath
    - `allow_env_var_aliases`: Allow/disallow using environment variables in name paths
    - `allow_env_vars []`: Allows only the specified environment variables to filter.

- Decrease SLOC count. For a project of this size, it shouldn't be 1K SLOC.
    There is a lot of duplicate code, and code that generally doesn't make sense/doesn't do what is intended.
    Maybe after the parser upgrade this will be halved or even reduced by a higher percent?

- More unit tests, more refactoring.

## Binary

The binary is still missing one key feature, namely the update and remove hooks:

For the shell hooks:

- The code is slightly messy and/or fragile at the moment and
    should be refactored in the future to be more robust.
- There are some problems with moving/removing files that are found across symlinks.
    The file isn't correctly detected across symlinks and thus isn't removed properly.

- `shortpath refresh`: Platform specific.
    Unsets all shortpath variables for the platform (bash, powershell), sets them again, and then refreshes the current shell
    with the new definitions.

- `shortpath clear [alias]`:
    - Clears the shortpath alias for the specified shortpath alias.
    - Prints out the cleared shortpath
    - Cross platform and works on both Linux \& Windows
- `shortpath clear [alias] -e`:
    - Clears only the environment variable

- Detailed manpage file for Linux users.

## Consider

For the shell hooks, consider:

- Should the majority of the logic moved into into the shortpaths binary instead,
    of in the shell scripts themselves?
    - Doing so may provide better cross platform functionality and/or performance.
    - If done, all clap arg parsing must be disabled and handled manually.
- Shortpaths could alternatively provide `mv` and/or `rm` commands that wrap
    native platform commands instead of providing an update hook.
    Examples: `sp mv`, `sp rm`.

- Consider using the `tracing` crate for logging.
