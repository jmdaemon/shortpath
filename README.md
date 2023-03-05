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

Combined with the shell update hooks **(Not Yet Implemented)**, its possible for the user to define a path and seamlessly
work without having to worry about updating aliases again.

> What happens if I one of my paths is still broken?

If the path is breakable, then you can make use of `shortpaths` additional feature:
- `shortpath resolve`: Lets you manually/automatically find and fix unreachable or broken shortpaths.

If you would prefer to work through your paths manually then `shortpaths` has you covered:
- `shortpath list`: Displays your current configuration
- `shortpath list [alias]`: Displays the path of the corresponding `shortpath`.

See [here](#usage) for more information on managing shortpaths.

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
helper scripts in `hooks` **Not Yet Implemented**.

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

#### Environment Variables

Environment variables are not as supported and are somewhat buggy.

- If an environment variable exists, with the shortpath variable containing the name of
    the shortpath of the same name, the envvar will replace the substring with itself
    thus breaking the path.
- Allowing any arbitrary envvar to be used in folding shortpaths is insecure.
    This should be modified to allow only specific customizable env vars such as
    `$profile`, the `XDG` variables, etc.

#### Export

- The `bash` and `powershell` exporter code has too much similarity with each other.
    Removing the trait and including a simpler method of dynamic dispatch will
    reduce code duplication between them.

#### Folding

- In `resolve_shortpath` the newly updated shortpath won't be folded after being updated.
- The shortpaths file isn't sorted after running add/remove/update commands.

- Eliminate as many duplicate and overlapping paths as possible.
    One big issue currently is that if there are many shortpaths with overlapping paths,
    there will be more breakages than if they relied on their respective GCD paths.

- There needs to be a way to exhaustively determine shortest paths given the current aliases such that:
    - Shortpath aliases have higher priority than environment variables
    - All shortpath aliases are folded exhaustively

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

## Binary

The binary is still missing one key feature, namely the update and remove hooks:

- `shortpath refresh`: Platform specific.
    Unsets all shortpath variables for the platform (bash, powershell), sets them again, and then refreshes the current shell
    with the new definitions.
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

- Consider `tracing` crate.
