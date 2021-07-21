# rpg-cli —your filesystem as a dungeon!

rpg-cli is a bare-bones [JRPG](https://en.wikipedia.org/wiki/JRPG)-inspired terminal game written in Rust. It can work as an alternative to `cd` where you randomly encounter enemies as you change directories.

![](rpg-cli.png)

Features:

* Character stats and leveling system.
* Automatic turn-based combat.
* Item and equipment support.
* 15+ enemy classes.
* Extensible player and enemy classes via configuration.
* Quests to-do list.
* Chests hidden in directories.
* Permadeath with item recovering.
* Run and bribe to escape battles.

## Installation

### From binary

Just download the binary for your platform (linux/macOS/windows) from the [GitHub releases page](https://github.com/facundoolano/rpg-cli/releases/latest).

### Using Cargo
Assuming you have [Rust and Cargo installed](https://doc.rust-lang.org/cargo/getting-started/installation.html#install-rust-and-cargo):

    $ cargo install --git https://github.com/facundoolano/rpg-cli --force --tag 0.5.0

The binary should be available as `rpg-cli` (assuming you have `~/.cargo/bin` in your `$PATH`).

### Using homebrew (macOS)
You can use homebrew to install the binary on macOS::

    $ brew install rpg-cli

### Using nixpkgs
If you use nix/nixos you can get rpg-cli from nixpkgs, either install it by adding it to your system config, installing it with `nix-env -i rpg-cli` or try it in a ephemeral shell with `nix-shell -p rpg-cli`.
Note that at the current time of writing, the package hasn't hit any of the channels yet. When you try it, check that it's in your channel.

### Using portage (Gentoo)
If you use Gentoo, you can get rpg-cli from portage:

    # emerge -av games-rpg/rpg-cli

## Shell integration (recommended)

The game is designed to integrate with common file system operations, such as changing directories or deleting files.
The most basic type of integration consists in wrapping rpg-cli in a shell function, such that the working directory is updated to match the hero's progress, effectively working as a `cd` alternative:

```sh
rpg () {
    rpg-cli "$@"
    cd "$(rpg-cli pwd)"
}
```

If you want to go all the way and *really* use it in place of `cd`:

```sh
cd () {
    rpg-cli cd "$@"
    builtin cd "$(rpg-cli pwd)"
}
```

Other commands like `rm`, `mkdir`, `touch`, etc. can also be aliased. Check [this example](shell/example.sh) and the [shell integration guide](shell/README.md) for more sophisticated examples, as well as their fish shell equivalents.

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

When running without parameters, as above, the hero status is printed (health points, accumulated experience, etc.). If you use the `cd` with a path as parameter, it will instruct the hero to move:

    ~ $ rpg cd dev/
    ~/dev $ rpg
        hero[1]@~/dev
        hp:[xxxxxxxxxx] 25/25
        xp:[----------] 0/30
        att:10   def:0   spd:5
        equip:{}
        item:{}
        0g

In this case, the hero moved to `~/dev`. Sometimes enemies will appear as you move through the directories,
and both characters will engage in battle:

    ~/dev $ rpg cd facundoolano/
       snake[1][xxxx]@~/dev/facundoolano
       snake[1][x---] -12hp
        hero[1][xxxx]  dodged!
       snake[1][----] -12hp
        hero[1][xxxx] +24xp +75g
        hero[1][xxxx][xxxx]@~/dev/facundoolano

Each character attacks in turn (the frequency being determined by their `speed` stat).
After taking an enemies hit, there's a chance to get a status effect, which will affect subsequent actions: hero attacks and moves.
Whenever you win a fight, your hero gains experience points and eventually raises its level, along with their other stats.

When you return to the home directory, the hero's health points are restored and status effects are removed:

    ~/dev/facundoolano/rpg-cli $ rpg cd ~
        hero[1][xxxx][xxxx]@home +20hp +healed

Also at the home directory, you can buy items and equipment:

    ~ $ rpg buy
        sword[1]    500g
        shield[1]   500g
        potion[1]   200g
        remedy      400g
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

The shortcut `rpg b p` would also work above. The item can then be used as `rpg use potion`.

Some directories have hidden treasure chests that you can find with `rpg ls`:

    ~ $ rpg ls
    📦  +potionx2

The `rpg todo` command will display a list of quest for your hero:

    ~ $ rpg todo
      □ buy a sword
      □ use a potion
      □ reach level 2
      ✔ win a battle

Each time you complete an item on the list, you will receive a reward. The quests renew as your level raises, so be sure to check often!

The further from home you move the hero, the tougher the enemies will get. If you go to far or too long without restoring your health, your hero is likely to die in battle, causing the game to restart at the home directory.

    ~ $ rpg cd ~/dev/facundoolano/rpg-cli/target/debug/examples/
         orc[1][xxxx]@~/dev/facundoolano/rpg-cli
        hero[1][x---] -20hp critical!
        hero[1][x---]  got burned 🔥
         orc[1][xxx-] -9hp
        hero[1][x---] -1hp 🔥
        hero[1][----] -16hp
        hero[1][----] 💀

Death is permanent: you can't save your progress and reload after dying, but if you take your new hero to the location of the previous one's death,
you can recover gold, items and equipment:

    ~ $ rpg cd - && rpg ls
    🪦 +potionx1 +75g


Try `rpg --help` for more options and check the [shell integration guide](shell/README.md) for ideas to adapt the game to your preferences.

## Customize character classes

The character class determines a character's initial stats and at what pace they increase when leveling up. By default, rpg-cli will use classes as defined by [this file](src/character/classes.yaml), but these definitions can be overridden by placing a YAML file with that same structure at `~/.rpg/classes.yaml`.

The `category` field is used to distinguish between player and enemy classes, and in the latter case how likely a given enemy class is likely to appear (e.g. `legendary` classes will appear less frequently, and only when far away from home).

The hero's class can be changed at the home directory using `rpg-cli class <name>`. If the hero is at level 1 it will effectively work as a character re-roll with fresh stats; at higher levels the stats are preserved and the class change will start taking effect on the next level increment.

## Troubleshooting

* The release binary for macOS [is not signed](https://github.com/facundoolano/rpg-cli/issues/27). To open it for the first time, right click on the binary and select "Open" from the menu.
