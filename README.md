# rpg-cli —your filesystem as a dungeon!

rpg-cli is a bare-bones [JRPG](https://en.wikipedia.org/wiki/JRPG)-inspired terminal game written in Rust. It can work as an alternative to `cd` where you randomly encounter enemies as you change directories.

![](rpg-cli.png)

Features:

* Automatic turn-based combat.
* 15+ enemy classes.
* Character stats and leveling system.
* Basic item and equipment support.
* Permadeath with item recovering.
* Run and bribe to escape battles.

## Setup

### Installing from binary

Just download the binary for your platform (linux/macOS/windows) from the [GitHub releases page](https://github.com/facundoolano/rpg-cli/releases/latest).

### Installing with Cargo
Assuming you have [Rust and Cargo installed](https://doc.rust-lang.org/cargo/getting-started/installation.html#install-rust-and-cargo):

    $ cargo install --git https://github.com/facundoolano/rpg-cli --force --branch main

The binary should be available as `rpg-cli` (assuming you have `~/.cargo/bin` in your `$PATH`).

### Installing with nixpkgs
If you use nix/nixos you can get rpg-cli from nixpkgs, either install it by adding it to your system config, installing it with `nix-env -i rpg-cli` or try it in a ephemeral shell with `nix-shell -p rpg-cli`.
Note that at the current time of writing, the package hasn't hit any of the channels yet. When you try it, check that it's in your channel.

### Shell integration (recommended)

The game is designed to integrate with common file system operations, such as changing directories or deleting files.

The most basic type of integration consists in wrapping rpg-cli in a shell function, such that the working directory is updated to track to the hero's progress, effectively working as a `cd` alternative:

```sh
rpg () {
    rpg-cli "$@"
    cd "$(rpg-cli pwd)"
}
```

Or, if you want to go all the way and *really* use it in place of `cd`:

```sh
cd () {
    rpg-cli cd "$@"
    builtin cd "$(rpg-cli pwd)"
}
```

Other commands like `rm`, `mkdir`, `touch`, etc. can also be aliased. Check the [shell integration guide](shell/README.md) for more sophisticated examples, as well as their fish shell equivalents.

### Troubleshooting

* The release binary for macOS [is not signed](https://github.com/facundoolano/rpg-cli/issues/27). To open it for the first time, right click on the binary and select "Open" from the menu.

## Usage

This example session assumes a basic `rpg` function as described in the previous section.

The first time you run the program, a new hero is created at the user's home directory.

    ~ $ rpg
        hero[1]@home
        hp:[xxxxxxxxxx] 25/25
        xp:[----------] 0/30
        att:10   def:0   spd:5
        equip:{}
        item:{}
        0g

When running without parameters, as above, the hero status is printed (health points, accumulated experience, etc.). If you pass in a path as parameter, that will instruct the hero to move:

    ~ $ rpg cd dev/
    ~/dev $ rpg
        hero[1]@~/dev
        hp:[xxxxxxxxxx] 25/25
        xp:[----------] 0/30
        att:10   def:0   spd:5
        equip:{}
        item:{}
        0g

In this case, the hero moved to `~/dev/facundoolano`. Sometimes enemies will appear as you move through the directories,
and both characters will engage in battle:

    ~/dev $ rpg cd facundoolano/
       snake[1][xxxx]@~/dev/facundoolano
       snake[1][x---] -12hp
        hero[1][xxxx]  dodged!
       snake[1][----] -12hp
        hero[1][xxxx] +24xp +75g
        hero[1][xxxx][xxxx]@~/dev/facundoolano

Each character attacks in turn (the frequency being determined by their `speed` stat).
Whenever you win a fight, your hero gains experience points and eventually raises its level, along with their other stats.

When you return to the home directory, the hero's health points are restored:

    ~/dev/facundoolano/rpg-cli $ rpg cd ~
        hero[1][xxxx][xxxx]@home +20hp

Also at the home directory, you can buy items and equipment:

    ~ $ rpg buy
        sword[1]    500g
        shield[1]   500g
        potion[1]   200g
        escape      1000g

        funds: 275g

    ~ $ rpg buy potion
    ~ $ rpg
        hero[3]@home
        hp:[xxxxxxxxxx] 37/37
        xp:[xx--------] 19/155
        att:13   def:0   spd:7
        equip:{}
        item:{potion[1]x1}
        75g

The shortcut `rpg b p` would also work above.

The further from home you move the hero, the tougher the enemies will get. If you go to far or too long without restoring your health your hero is likely to die in battle, causing the game to restart.

    ~ $ rpg cd ~/dev/facundoolano/rpg-cli/target/debug/examples/
         orc[1][xxxx]@~/dev/facundoolano/rpg-cli
        hero[1][x---] -20hp critical!
         orc[1][xxx-] -9hp
        hero[1][----] -16hp
        hero[1][----] 💀

Death is permanent: you can't save your progress and reload after dying, but if you take your new hero to the location of the previous one's death,
you can recover gold, items and equipment:

    ~ $ rpg ~/dev/facundoolano/rpg-cli/
       hero[🪦]@~/dev/facundoolano/rpg-cli/ +potionx1 +75g


Try `rpg --help` for more options.
