# rpg-cli —your filesystem as a dungeon!

rpg-cli is a bare-bones [JRPG](https://en.wikipedia.org/wiki/JRPG)-inspired terminal game written in Rust. It can work as an alternative to `cd` where you randomly encounter enemies as you change directories.

![](rpg-cli.png)

## Installation

### From source
Assuming you have [Rust and Cargo installed](https://doc.rust-lang.org/cargo/getting-started/installation.html#install-rust-and-cargo), clone the repo and run:

    $ cargo build --release

The binary will be available at `target/release/rpg-cli`.

Check the [rpg.sh](./rpg.sh) script for an example of how to setup the game with a shell alias to work as `cd` (so the working directory is updated according to the hero's progress).

## Usage

The first time you run the program, a new hero is created at the user's home directory.

    ~ $ rpg
        hero[1]@home
        hp:[xxxxxxxxxx] 25/25
        xp:[----------] 0/30
        str:10   spd:5   100g
        equip:{sword, shield}
        item:{}

When running without parameters, as above, the hero status is printed (health points, accumulated experience, etc.). If you pass in a path as parameter, that will instruct the hero to move:

    ~ $ rpg dev/facundoolano/
    ~/dev/facundoolano $ rpg
        hero[1]@~/dev/facundoolano
        hp:[xxxxxxxxxx] 25/25
        xp:[----------] 0/30
        str:10   spd:5   100g
        equip:{sword, shield}
        item:{}

In this case, the hero moved to `~/dev/facundoolano`. Sometimes enemies will appear as you move through the directories,
and both characters will engage in battle:

    ~/dev/facundoolano $ rpg rpg-cli/target/debug/
        enemy[1][xxxx][----]@~/dev/facundoolano/rpg-cli
        enemy[1][xx--][----]@~/dev/facundoolano/rpg-cli -10hp
         hero[1][xxx-][----]@~/dev/facundoolano/rpg-cli -9hp
        enemy[1][x---][----]@~/dev/facundoolano/rpg-cli -8hp
         hero[1][xx--][----]@~/dev/facundoolano/rpg-cli -9hp
        enemy[1][----][----]@~/dev/facundoolano/rpg-cli -8hp
         hero[1][xx--][xxxx]@~/dev/facundoolano/rpg-cli +26xp +100g

Each character attacks in turn (the frequency being determined by their `speed` stat).
Whenever you win a fight, your hero gains experience points and eventually raises its level, along with their other stats.

When you return to the home directory, the hero's health points are restored:

    ~/dev/facundoolano/rpg-cli $ rpg ~
        hero[1][xxxx][xxxx]@home +20hp

The further from home you move the hero, the tougher the enemies will get. If you go to far or too long without restoring your health your hero is likely to die in battle, causing the game to reset.

    ~ $ rpg dev/facundoolano/rpg-cli/target/debug/
        enemy[6][xxxx][----]@~/dev/facundoolano/rpg-cli/target/debug
         hero[2][x---][xxx-]@~/dev/facundoolano/rpg-cli/target/debug -16hp
        enemy[6][xxxx][----]@~/dev/facundoolano/rpg-cli/target/debug -8hp
         hero[2][----][xxx-]@~/dev/facundoolano/rpg-cli/target/debug -18hp
         hero[2][----][xxx-]@~/dev/facundoolano/rpg-cli/target/debug 💀
    ~ $ rpg
        hero[1]@home
        hp:[xxxxxxxxxx] 25/25
        xp:[----------] 0/30
        str:10   spd:5   100g
        equip:{sword, shield}
        item:{}
