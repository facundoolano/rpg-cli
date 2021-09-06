#!/bin/bash
# An example setup to use the game in the bash shell. Check the guide for more options.

# FIXME override this with your own path to rpg-cli
RPG=/your/path/to/facundoolano/rpg-cli/target/release/rpg-cli

# Use the rpg as the command to do non fs related tasks such as print
# status and buy items.
rpg () {
    $RPG "$@"
    sync_rpg
}

# Try to move the hero to the given destination, and cd match the shell pwd
# to that of the hero's location:
# - the one supplied as parameter, if there weren't any battles
# - the one where the battle took place, if the hero wins
# - the home dir, if the hero dies
cd () {
    $RPG cd "$@"
    sync_rpg
}

# Generate dungeon levels on the fly and look for treasures while moving down.
#  Will start by creating dungeon/1 at the current directory, and /2, /3, etc.
#  on subsequent runs.
dn () {
    current=$(basename $PWD)
    number_re='^[0-9]+$'

    if [[ $current =~ $number_re ]]; then
        next=$(($current + 1))
        command mkdir -p $next && cd $next && rpg ls
    elif [[ -d 1 ]] ; then
        cd 1 && rpg ls
    else
        command mkdir -p dungeon/1 && cd dungeon/1 && rpg ls
    fi
}

# This helper is used to make the pwd match the tracked internally by the game
sync_rpg () {
    builtin cd "$($RPG pwd)"
}
