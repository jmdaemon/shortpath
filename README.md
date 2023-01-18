# Shortpaths

Better file path aliases.

<!--Resilient alias file paths that don't break.-->

How it works:

1. Define an alias for a currently existing path
2. The moment that directory is deleted/moved
3. Run `shortpaths auto index`

If any paths are missing/unreachable, go up one parent directory and search for directory name

If found, set shortpath to found path, and echo new shortpath
Else if not found (and our currently searched directory is $HOME or /)
    Print 'Shortpath $alias_name : $old_path_name is missing'

Shortpaths will also generate additional shell completions for you that you can
load with `source ~/.config/shortpaths/shell/completions`.


<!--Security:-->
<!--- The shortpaths file can only be edited by the current user.-->
<!--- Certain shortpaths can be refrained from being edited/removed or checked with the `const: true` property-->

<!--Commands-->
<!--- add, remove, update, check-->

<!--These commands will correspond to various operations such as:-->
<!--mv -> update-->
<!--rm -> remove-->

Benefits over using shell variables

## Usage

```bash
# Adds a new shortpath
shortpath add "name" "path"

# Removes a shortpath
shortpath remove -n "name" # Remove by name
shortpath remove -p "path" # Remove by path

# Checks all available shortpaths for missing/unreachable paths
# If the path is missing, warn the user
shortpath check

# Like check but performs the shortpath auto migration
shortpath autoindex

# Update a shortpath
shortpath update "current_name" -n "new_name" # Renames shortpath
shortpath update "current_name" -p "new_path" # Update shortpath directory

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

## Features

- Resilience: Shortpaths are designed to almost never break/be unreachable.
- Nested shortpath definitions. Shortpaths can contain other shortpaths in their filename.
    This further improves resilience by making it efficient to modify one shortpath.
    If a shortpath is found to be unreachable, then it can be easily remedied by updating one shortpath instead of many files paths for nested files.
- Universal: Shortpaths can export shell alias completions and also be embedded for use in applications.
- Ease of Use: Adding new shortpaths is as easy as `shortpath add [name] [path]`
- Centralization: All your shortpaths are stored in one file for you to view
- Better Security: Your shortpaths config is only editable by you. In the case of
    bash completions, you can be sure the completions file generated by shortpaths is secure.

