#!/bin/bash

# Thoughts:
# Should all the arg parsing logic be moved into into the shortpaths binary instead?
# - If so, all clap argument formatting/parsing must be disabled and handled manually.

# Find and update any entry in the shortpaths config that contains "$src"
mv() {
    src_was_set=0
    dest_was_set=0

    for var in "$@" do
        # Set dest
        if [[ "$var" == -* && (src_was_set == 0) ]]; then
            src="$var"
            src_was_set=1
        fi

        # Set source
        if [[ "$dest" == -* && (dest_was_set == 0) ]]; then
            dest="$var"
            dest_was_set=1
        fi

        # Do the move
        if [[ (src_was_set == 1) && (dest_was_set == 1) ]]; then
            shortpaths move_hook "$src" "$dest"

            # Pass all the arguments
            /usr/bin/mv $@
        fi
    done
}

# Find and remove any entry in the shortpaths config that contains "$src"
rm() {
    src_was_set=0

    for var in "$@" do
        # Set dest
        if [[ "$var" == -* && (src_was_set == 0) ]]; then
            src="$var"
            src_was_set=1
        fi

        if [[ (src_was_set == 1) && (dest_was_set == 1) ]]; then
            shortpaths remove_hook "$src"

            # Pass all the arguments
            /usr/bin/rm $@
        fi
    done
}
