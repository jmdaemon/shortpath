#!/bin/bash

# Thoughts:
# Should all the arg parsing logic be moved into into the shortpaths binary instead?
# - If so, all clap argument formatting/parsing must be disabled and handled manually.

# Find and update any entry in the shortpaths config that contains "$src"
mv() {
    src_was_set=0
    dest_was_set=0
    src=""
    dest=""

    for var in $@; do
        # Get the valid src path
        if [[ (! "$var" =~ "^-") && (! -z "$var") && (src_was_set -eq 0) ]]; then
            src_was_set=1
            src="$var"
            unset var
        fi

        # Get the valid dest path
        if [[ (! "$var" =~ "^-") && (! -z "$var") && (dest_was_set -eq 0) ]]; then
            dest_was_set=1
            dest="$var"
            unset var
        fi

        # Move the files
        if [[ (src_was_set -eq 1) && (dest_was_set -eq 1) ]]; then
            shortpath -v hook move "$src" "$dest"
            # Move the files only if the shortpath hook passed
            if [[ $? -eq 0 ]]; then
                /usr/bin/mv "$src" "$dest"
            fi
        fi
    done
}

# Find and remove any entry in the shortpaths config that contains "$src"
rm() {
    src_was_set=0

    for var in "$@"; do
        # Set dest
        if [[ "$var" == -* && (src_was_set == 0) ]]; then
            src="$var"
            src_was_set=1
        fi

        if [[ (src_was_set == 1) && (dest_was_set == 1) ]]; then
            shortpaths hook remove "$src"

            # Pass all the arguments
            /usr/bin/rm $@
        fi
    done
}
