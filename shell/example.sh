#!/bin/bash
# An example setup to use the game in the bash shell. Check the guide for more options.

# Use the rpg as the command to do non fs related tasks such as print
# status and buy items.
# TODO override this with your own path to rpg-cli
alias rpg=/your/path/to/facundoolano/rpg-cli/target/release/rpg-cli

# Try to move the hero to the given destination, and cd match the shell pwd
# to that of the hero's location:
# - the one supplied as parameter, if there weren't any battles
# - the one where the battle took place, if the hero wins
# - the home dir, if the hero dies
cd () {
    rpg cd "$@"
    builtin cd "$(rpg pwd)"
}

# When invoking ls without additional args, suffix the output with
# any chest or tombstone found at the current location
ls () {
    command ls "$@"
    if [ $# -eq 0 ] ; then
        rpg cd -f .
	rpg ls
    fi
}


# For other file related commands, force-move the hero to the current working dir
# then potentially initiate a battle. Only if the hero wins or there's no battle
# the required command is actually run
battle="rpg cd -f . && rpg battle"
alias rm="$battle && rm"
alias rmdir="$battle && rmdir"
alias mkdir="$battle && mkdir"
alias touch="$battle && touch"
alias mv="$battle && mv"
alias cp="$battle && cp"
alias chown="$battle && chown"
alias chmod="$battle && chmod"
