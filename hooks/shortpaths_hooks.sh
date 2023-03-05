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

    echo $@
    for var in $@; do
        echo $var
        # Set dest
        # If the variable 
        if [[ (! "$var" =~ "^-") && (! -z "$var") && (src_was_set -eq 0) ]]; then
            src_was_set=1
            src="$var"
            unset var
            echo "Setting src: $src"
            #shift
            #shift
        fi

        # Set source
        if [[ (! "$var" =~ "^-") && (! -z "$var") && (dest_was_set -eq 0) ]]; then
            dest_was_set=1
            dest="$var"
            #shift
            unset var
            echo "Setting dest: $dest"
        fi

        # Do the move
        #if [[ (src_was_set == 1) && (dest_was_set == 1) ]]; then
        if [[ (src_was_set -eq 1) && (dest_was_set -eq 1) ]]; then
            shortpath -v hook move "$src" "$dest"
            #echo "shortpaths -v hook move \"$src\" \"$dest\""

            # Pass all the arguments
            /usr/bin/mv "$src" "$dest"
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
