# rpg-cli —your filesystem as a dungeon!

rpg-cli is a bare-bones [JRPG](https://en.wikipedia.org/wiki/JRPG)-inspired terminal game written in Rust. It can work as an alternative to `cd` where you randomly encounter enemies as you change directories.

![](rpg-cli.png)

## Setup

### Installing from binary

Just download the binary for your platform (linux/macOS) from the [GitHub releases page](https://github.com/facundoolano/rpg-cli/releases/latest).

### Installing from source
Assuming you have [Rust and Cargo installed](https://doc.rust-lang.org/cargo/getting-started/installation.html#install-rust-and-cargo), clone the repo and run:

    $ cargo build --release

The binary will be available at `target/release/rpg-cli`.

### Use as a cd replacement (recommended)

Once the binary is installed with one of the methods described above, it can be wrapped on a shell function or alias
so the working directory is updated to track to the hero's progress. You can set that up by adding something like this to your `.bashrc` (with the `RPG_CLI` variable pointing to the binary location):

```sh
RPG_CLI=/Users/facundo/dev/facundoolano/rpg-cli/target/release/rpg-cli
rpg () {
   $RPG_CLI "$@"
   cd $($RPG_CLI --pwd)
}
```

## Usage

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

    ~ $ rpg dev/
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

    ~/dev $ rpg facundoolano/

       snake[1][xxxx]@~/dev/facundoolano

       snake[1][x---] -12hp
        hero[1][xxxx]  dodged!
       snake[1][----] -12hp

        hero[1][xxxx][xxxx]@~/dev/facundoolano +24xp +75g

Each character attacks in turn (the frequency being determined by their `speed` stat).
Whenever you win a fight, your hero gains experience points and eventually raises its level, along with their other stats.

When you return to the home directory, the hero's health points are restored:

    ~/dev/facundoolano/rpg-cli $ rpg ~
        hero[1][xxxx][xxxx]@home +20hp

Also at the home directory, you can buy items and equipment:

    ~ $ rpg --shop

        sword[1]    500g
        shield[1]   500g
        potion[1]   200g
        escape      1000g

        funds: 275g

    ~ $ rpg --shop potion
    ~ $ rpg

        hero[3]@home
        hp:[xxxxxxxxxx] 37/37
        xp:[xx--------] 19/155
        att:13   def:0   spd:7
        equip:{}
        item:{potion[1]x1}
        75g

The shorcut `rpg -s p` would also work above.

The further from home you move the hero, the tougher the enemies will get. If you go to far or too long without restoring your health your hero is likely to die in battle, causing the game to restart.

    ~ $ rpg ~/dev/facundoolano/rpg-cli/target/debug/examples/

         orc[1][xxxx]@~/dev/facundoolano/rpg-cli

        hero[1][x---] -20hp critical!
         orc[1][xxx-] -9hp
        hero[1][----] -16hp

        hero[1][----][----]@~/dev/facundoolano/rpg-cli 💀

Death is permanent: you can't save your progress and reload after dying, but if you take your new hero to the location of the previous one's death,
you can recover gold, items and equipment:

    ~ $ rpg ~/dev/facundoolano/rpg-cli/

        🪦 @~/dev/facundoolano/rpg-cli/

        +potionx1
        +75g


Try `rpg --help` for more options.
