# rpg-cli —your filesystem as a dungeon!

rpg-cli is a minimalist [computer RPG](https://en.wikipedia.org/wiki/Role-playing_video_game) written in Rust. Its command-line interface can be used as a `cd` replacement where you randomly encounter enemies as you change directories.

![](rpg-cli.png)

Features:

* Character stats and leveling system.
* Automatic turn-based combat.
* Item and equipment support.
* Warrior, thief and mage player classes.
* 15+ Enemy classes.
* Extensible player and enemy classes via configuration.
* Permadeath with item recovering.
* Quests to-do list.
* Chests hidden in directories.

## Installation

### From binary

Just download the binary for your platform (linux/macOS/windows) from the [GitHub releases page](https://github.com/facundoolano/rpg-cli/releases/latest).

### Using Cargo
Assuming you have [Rust and Cargo installed](https://doc.rust-lang.org/cargo/getting-started/installation.html#install-rust-and-cargo):

    $ cargo install --git https://github.com/facundoolano/rpg-cli --force --tag 1.0.0-beta

The binary should be available as `rpg-cli` (assuming you have `~/.cargo/bin` in your `$PATH`).

### Other installation methods
<details>
  <summary>Show details</summary>

#### Homebrew (macOS)
You can use homebrew to install the binary on macOS::

    $ brew install rpg-cli

#### Nixpkgs
If you use nix/nixos you can get rpg-cli from nixpkgs, either install it by adding it to your system config, installing it with `nix-env -i rpg-cli` or try it in a ephemeral shell with `nix-shell -p rpg-cli`.
Note that at the current time of writing, the package hasn't hit any of the channels yet. When you try it, check that it's in your channel.

#### Portage (Gentoo)
If you use Gentoo, you can get rpg-cli from portage:

    # emerge -av games-rpg/rpg-cli

#### Pacman (Arch Linux)

rpg-cli can be installed from the [community repository](https://archlinux.org/packages/community/x86_64/rpg-cli/) for Arch Linux:

    $ pacman -S rpg-cli
</details>

## Shell integration

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

## Gameplay

This example session assumes a basic `rpg` function as described in the previous section.

### Character setup
The first time you run the program, a new hero is created at the user's home directory.

    ~ $ rpg
     warrior[1]@home
        hp:[xxxxxxxxxx] 48/48
        mp:[----------] 0/0
        xp:[----------] 0/30
        att:10   mag:0   def:0   spd:10
        equip:{}
        item:{}
        0g

When running without parameters, as above, the hero status is printed (health points, accumulated experience, etc.).
The stats are randomized: if you run `rpg reset` you will get a slightly different character every time:

    ~ $ rpg reset; rpg
     warrior[1]@home
        hp:[xxxxxxxxxx] 50/50
        mp:[----------] 0/0
        xp:[----------] 0/30
        att:13   mag:0   def:0   spd:12
        equip:{}
        item:{}
        0g

You can also pick a different class (default options are `warrior`, `thief` and `mage`, but [more can be added](#customize-character-classes)).
For example, the `mage` class enables magic attacks:

    ~ $ rpg class mage; rpg
        mage[1]@home
        hp:[xxxxxxxxxx] 32/32
        mp:[xxxxxxxxxx] 12/12
        xp:[----------] 0/30
        att:3   mag:27   def:0   spd:9
        equip:{}
        item:{}
        0g

### Movement and battles
If you use the `cd` subcommand with a path as parameter, it will instruct the hero to move:

    ~ $ rpg cd dev/
    ~/dev $ rpg
        warrior[1]@~/dev
        hp:[xxxxxxxxxx] 47/47
        mp:[----------] 0/0
        xp:[----------] 0/30
        att:10   mag:0   def:0   spd:12
        equip:{}
        item:{}
        0g

In this case, the warrior moved to `~/dev`. Sometimes enemies will appear as you move through the directories,
and both characters will engage in battle:

    ~/dev $ rpg cd facundoolano/
       snake[3][xxxx][----]@~/dev/facundoolano
       snake[3][xxx-] -10hp
     warrior[1][xxxx] -8hp
       snake[3][xxx-] -9hp
     warrior[1][xxx-] -10hp
       snake[3][x---] -12hp
     warrior[1][xx--] -9hp
       snake[3][----] -14hp
     warrior[3][xxx-] +117xp ++level +275g
     warrior[3][xxx-][----][x---]@~/dev/facundoolano

Each character attacks in turn (the frequency being determined by their `spd` stat).
Whenever you win a fight, your hero gains experience points and eventually raises its level, along with its other stats.

When you return to the home directory, the hero's health points are restored and status effects are removed:

    ~/dev/facundoolano/rpg-cli $ rpg cd ~
        warrior[3][xxxx][----][x---]@home +27hp

The further from home you move the hero, the tougher the enemies will get. If you go to far or too long without restoring your health, your hero is likely to die in battle, causing the game to restart at the home directory.

    ~ $ rpg cd ~/dev/facundoolano/rpg-cli/target/debug/examples/
      zombie[3][xxxx][----]@~/dev/facundoolano/rpg-cli/target/debug
      zombie[3][xxxx] -14hp
      warrior[1][xxx-] -14hp
      zombie[3][xxx-] -16hp
      warrior[1][xxx-] -11hp
      zombie[3][xx--] -16hp
      warrior[1][xx--] -9hp
      zombie[3][xx--] -15hp
      warrior[1][x---] -9hp
      zombie[3][x---] -12hp
      warrior[1][----] -20hp critical!
      warrior[1][----] 💀

Death is permanent: you can't save your progress and reload after dying, but if you take your new hero to the location of the previous one's death,
you can recover gold, items and equipment:

    ~ $ rpg cd ~/dev/facundoolano/rpg-cli/target/debug/
    🪦 +potionx1 +275g

### Items and equipment

In addition to winning items as battle rewards, some directories have hidden treasure chests that you can find with `rpg ls`:

    ~ $ rpg ls
    📦  +potionx2

Finally, some items can be bought at the game directory running `rpg buy`:

    ~ $ rpg buy
        sword[1]    500g
        shield[1]   500g
        potion[1]   200g
        remedy      400g
        escape      1000g

        funds: 275g
    ~ $ rpg buy potion
       -200g +potionx1

The shortcut `rpg b p` would also work above. An item can be described with the `stat` subcommand and used with `use`:

    ~ $ rpg stat potion
    potion[1]: restores 25hp
    ~ $ rpg use potion
     warrior[3][xxxx] +25hp potion

### Quests and late game

The `rpg todo` command will display a list of quest for your hero:

    ~ $ rpg todo
      □ buy a sword
      ✔ use a potion
      ✔ reach level 2
      ✔ win a battle

Each time you complete an item on the list, you will receive a reward. The quests renew as your level raises, so be sure to check often!

The game difficulty increases as you go deeper in the dungeon; to raise your level, encounter the tougher enemies, find the rarest items
and complete all the quests, it's necessary to go as far as possible from the `$HOME` directory. One option to ease the gameplay
is to [use a shell function](https://github.com/facundoolano/rpg-cli/blob/main/shell/README.md#arbitrary-dungeon-levels) that creates directories "on-demand".

Try `rpg --help` for more options and check the [shell integration guide](shell/README.md) for ideas to adapt the game to your preferences.

## Customize character classes

The character class determines a character's initial stats and at what pace they increase when leveling up. By default, rpg-cli will use classes as defined by [this file](src/character/classes.yaml), but these definitions can be overridden by placing a YAML file with that same structure at `~/.rpg/classes.yaml`.

The `category` field is used to distinguish between player and enemy classes, and in the latter case how likely a given enemy class is likely to appear (e.g. `legendary` classes will appear less frequently, and only when far away from home).

The hero's class can be changed at the home directory using `rpg-cli class <name>`. If the hero is at level 1 it will effectively work as a character re-roll with fresh stats; at higher levels the stats are preserved and the class change will start taking effect on the next level increment.

## Troubleshooting

* The release binary for macOS [is not signed](https://github.com/facundoolano/rpg-cli/issues/27). To open it for the first time, right click on the binary and select "Open" from the menu.
