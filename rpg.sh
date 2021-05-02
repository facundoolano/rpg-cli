#!/usr/bin/env bash
# Example script to use rpg-cli as a "spiced" alias for `cd`
# e.g. put this in your .bashrc:
# alias rpg=". ~/dev/facundoolano/rpg-cli/rpg.sh"

rpg=~/dev/facundoolano/rpg-cli/target/release/rpg-cli
$rpg "$@"
dest=$($rpg --pwd)
cd $dest
