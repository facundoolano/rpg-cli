# Shell integration

To get the most out of rpg-cli, it is suggested to define aliases or wrapper functions so the game can be integrated into a regular shell session, with enemies appearing along the way.

This guide describes the basic building blocks to write such functions and shows some examples.

## Basic `cd` alternative

The default rpg-cli command works as `cd`, changing the hero's location from
one directory to another. Since the program itself can't affect your shell session,
you need to write a function so the working directory is changed to match that of the hero:

```sh
rpg () {
    rpg-cli "$@"
    cd "$(rpg-cli pwd)"
}
```

This assumes `rpg-cli` is in your path, update with the specific location if not. You can define it directly in your current session, add it to `~/.bashrc`, source it from another script, etc.

If you use fish shell, update `~/.config/fish/config.fish` instead:

```fish
function rpg
    rpg-cli $argv
    cd (rpg-cli pwd)
end
```

## Full `cd` override

If you like having enemies popping up while using `cd`, you can override that instead of using a separate function:

```sh
cd () {
    rpg-cli cd "$@"
    builtin cd "$(rpg-cli pwd)"
}
```

## Custom integration commands

To better adapt for different usage patterns, finer-grained commands are provided:

* `rpg-cli cd --force <path>` will set the hero's location to `<path>` without initiating battles.
* `rpg-cli pwd` will print the hero's current location.
* `rpg-cli battle` will initiate a battle with a probability that changes based on the distance from home. If the battle is lost the exit code of the program will be non-negative.
* `rpg-cli stat --quiet` will return hero stats in a succinct format.
* `rpg-cli stat --plain` will return hero stats as tab separated fields, to facilitate parsing (e.g. to integrate to the prompt).

## Prevent intermediate battles

Note that the logic of the default rpg command is this: the hero moves one directory at a time, and enemies can appear at each step:

* If the hero dies, the game is restarted and you go back home.
* If the hero wins the battle, it will stop at the battle's location instead of keep moving to the initial destination. The rationale for this behavior is that you may need to adjust your strategy after each battle: use a potion, return home, try to escape battles, etc.

Having `cd` not consistently set the pwd to the intended destination may not be acceptable if the program is used casually while doing other work.
A better alternative for this usage pattern is enabled by the other integration commands, for example:

```sh
cd () {
    builtin cd "$@"
    rpg-cli cd -f .
    rpg-cli battle
}
```

## Aliasing other commands

Another way to use rpg-cli is to initiate battles when attempting to execute file-modifying operations. Only when the battle is won the operation is allowed:

```sh
alias rpg-battle="rpg-cli cd -f . && rpg-cli battle"

alias rm="rpg-battle && rm"
alias rmdir="rpg-battle && rmdir"
alias mkdir="rpg-battle && mkdir"
alias touch="rpg-battle && touch"
alias mv="rpg-battle && mv"
alias cp="rpg-battle && cp"
alias chown="rpg-battle && chown"
alias chmod="rpg-battle && chmod"
```

## ls `cd` override

The `rpg-cli ls` command looks for chests at the current location.
It can be integrated to the regular ls like this:

``` sh
ls () {
    command ls "$@"
    if [ $# -eq 0 ] ; then
        rpg cd -f .
        rpg ls
    fi
}
```
