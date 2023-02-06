#!/bin/bash

# Used to differentiate arguments passed vs options passed
# Match anything but the passes options
# Usage: nmatch_options "${options[@]}"
#nmatch_options() {
    #options="$1"
    #args=[]
    #opts=[]
    #case in
    #esac
#}

mv() {
    #src=""
    #src=""
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
            # Finds any entry in the shortpaths config that contains
            #shortpaths update_hook "$src" "$dest"
            shortpaths move_hook "$src" "$dest"

            /usr/bin/mv $@
        fi
    done
}

rm() {
    #src=""
    #src=""
    src_was_set=0

    for var in "$@" do
        # Set dest
        if [[ "$var" == -* && (src_was_set == 0) ]]; then
            src="$var"
            src_was_set=1
        fi

        if [[ (src_was_set == 1) && (dest_was_set == 1) ]]; then
            # Finds any entry in the shortpaths config that contains "$src"
            #shortpaths update_hook "$src" "$dest"
            shortpaths remove_hook "$src"

            /usr/bin/rm $@
        fi
    done
}
