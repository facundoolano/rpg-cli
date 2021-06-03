use game::Game;

mod character;
mod game;
mod item;
mod location;
mod log;
mod randomizer;

use crate::location::Location;
use clap::{crate_version, Clap};

/// Your filesystem as a dungeon!
#[derive(Clap)]
#[clap(version = crate_version!(), author = "Facundo Olano <facundo.olano@gmail.com>")]
struct Opts {
    /// Moves the hero to the supplied destination.
    destination: Option<String>,

    /// Print the hero's stats
    #[clap(short, long)]
    stat: bool,

    /// Prints the hero's current location
    #[clap(long)]
    pwd: bool,

    /// Resets the current game.
    #[clap(long)]
    reset: bool,

    /// Attempt to avoid battles by running away.
    #[clap(long)]
    run: bool,

    /// Attempt to avoid battles by bribing the enemy.
    #[clap(long)]
    bribe: bool,

    /// Buys an item from the shop.
    /// If name is omitted lists the items available for sale.
    #[clap(short, long)]
    buy: bool,

    /// Uses an item from the inventory.
    #[clap(name = "use", short, long)]
    item: bool,

    /// Move the hero's to a different location without spawning enemies.
    #[clap(long)]
    mv: Option<String>,

    /// Potentially spawns an enemy in the current directory.
    #[clap(long)]
    battle: bool,
}

fn main() {
    let opts: Opts = Opts::parse();

    let mut game = Game::load().unwrap_or_else(|_| Game::new());
    let mut exit_code = 0;

    if opts.stat {
        log::status(&game);
    } else if opts.pwd {
        println!("{}", game.location.path_string());
    } else if let Some(dest) = opts.mv {
        mv(&mut game, &dest);
    } else if opts.reset {
        game.reset()
    } else if opts.buy {
        // when -s flag is provided, the positional argument is assumed to be an item
        shop(&mut game, &opts.destination);
    } else if opts.item {
        // when -i flag is provided, the positional argument is assumed to be an item
        item(&mut game, &opts.destination);
    } else if opts.battle {
        exit_code = battle(&mut game, opts.run, opts.bribe);
    } else {
        // when omitting the destination, go to home to match `cd` behavior
        let dest = opts.destination.unwrap_or_else(|| String::from("~"));
        exit_code = go_to(&mut game, &dest, opts.run, opts.bribe);
    }

    game.save().unwrap();
    std::process::exit(exit_code);
}

/// Main command, attempt to move the hero to the supplied location,
/// possibly engaging in combat along the way.
fn go_to(game: &mut Game, dest: &str, run: bool, bribe: bool) -> i32 {
    let mut exit_code = 0;
    if let Ok(dest) = Location::from(&dest) {
        if let Err(game::Error::GameOver) = game.go_to(&dest, run, bribe) {
            game.reset();
            exit_code = 1;
        }
        log::short_status(&game);
    } else {
        println!("No such file or directory");
        exit_code = 1
    }
    exit_code
}

/// Potentially run a battle at the current location, independently from
/// the hero's movement.
fn battle(game: &mut Game, run: bool, bribe: bool) -> i32 {
    let mut exit_code = 0;
    // FIXME duplication here
    if let Some(mut enemy) = game.maybe_spawn_enemy() {
        if let Err(game::Error::GameOver) = game.maybe_battle(&mut enemy, run, bribe) {
            game.reset();
            exit_code = 1;
        }
    }
    log::short_status(&game);
    exit_code
}

/// Override the hero's current location.
/// Intended for finer-grained shell integration.
fn mv(game: &mut Game, dest: &str) {
    if let Ok(dest) = Location::from(&dest) {
        game.location = dest;
    } else {
        println!("No such file or directory");
        std::process::exit(1);
    }
}

/// Buy an item from the shop or list the available items if no item name is provided.
/// Shopping is only allowed when the player is at the home directory.
fn shop(game: &mut Game, item_name: &Option<String>) {
    if game.location.is_home() {
        if let Some(item_name) = item_name {
            let item_name = sanitize(item_name);
            match item::shop::buy(game, &item_name) {
                Err(item::shop::Error::NotEnoughGold) => {
                    println!("Not enough gold.")
                }
                Err(item::shop::Error::ItemNotAvailable) => {
                    println!("Item not available.")
                }
                Ok(()) => {}
            }
        } else {
            item::shop::list(game);
        }
    } else {
        println!("Shop is only allowed at home.")
    }
}

/// Use an item from the inventory or list the inventory contents if no item name is provided.
fn item(game: &mut Game, item_name: &Option<String>) {
    if let Some(item_name) = item_name {
        let item_name = sanitize(item_name);
        if let Err(game::Error::ItemNotFound) = game.use_item(&item_name) {
            println!("Item not found.");
        }
    } else {
        println!("{}", log::format_inventory(&game));
    }
}

/// Return a clean version of an item/equipment name, including aliases
fn sanitize(name: &str) -> String {
    let name = name.to_lowercase();
    let name = match name.as_str() {
        "p" | "potion" => "potion",
        "e" | "escape" => "escape",
        "sw" | "sword" => "sword",
        "sh" | "shield" => "shield",
        n => n,
    };
    name.to_string()
}
