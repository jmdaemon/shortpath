#!/bin/bash

# Find and update any entry in the shortpaths config containing "$src"
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

# Find and remove any entry in the shortpaths config containing the filepath
rm() {
    removed=()
    for var in $@; do
        if [[ (! "$var" =~ "^-") && (! -z "$var") ]]; then
            removed+="$var"
        fi
    done

    if [[ ! -z $removed ]]; then
        shortpath hook remove ${removed[@]}
        # Move the files only if the shortpath hook passed
        if [[ $? -eq 0 ]]; then
            /usr/bin/rm $@
        fi
    fi
}
