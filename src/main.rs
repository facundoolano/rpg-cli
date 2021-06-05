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
    /// Potentially spawns an enemy in the current directory.
    #[clap(subcommand)]
    cmd: Option<Command>,
}

// FIXME check if the {} is actually necessary
#[derive(Clap)]
enum Command {
    /// Moves the hero to the supplied destination.
    #[clap(name = "cd")]
    ChangeDir {
        destination: Option<String>,

        /// Attempt to avoid battles by running away.
        #[clap(long)]
        run: bool,

        /// Attempt to avoid battles by bribing the enemy.
        #[clap(long)]
        bribe: bool,

        /// Move the hero's to a different location without spawning enemies.
        #[clap(short, long)]
        force: bool,
    },

    /// Resets the current game.
    Reset {},

    /// Buys an item from the shop.
    /// If name is omitted lists the items available for sale.
    Buy { item: Option<String> },

    /// Uses an item from the inventory.
    Use { item: Option<String> },

    /// Prints the hero's current location
    #[clap(name = "pwd")]
    PrintWorkDir {},

    Battle {
        /// Attempt to avoid battles by running away.
        #[clap(long)]
        run: bool,

        /// Attempt to avoid battles by bribing the enemy.
        #[clap(long)]
        bribe: bool,
    },
}

fn main() {
    let opts: Opts = Opts::parse();

    let mut game = Game::load().unwrap_or_else(|_| Game::new());
    let mut exit_code = 0;

    if opts.cmd.is_none() {
        log::status(&game);
    } else if let Some(Command::PrintWorkDir {}) = opts.cmd {
        println!("{}", game.location.path_string());
    } else if let Some(Command::ChangeDir {
        destination,
        run,
        bribe,
        force: false,
    }) = opts.cmd
    {
        // when omitting the destination, go to home to match `cd` behavior
        let dest = destination.unwrap_or_else(|| String::from("~"));
        exit_code = go_to(&mut game, &dest, run, bribe);
    } else if let Some(Command::ChangeDir {
        destination,
        force: true,
        ..
    }) = opts.cmd
    {
        // FIXME can this default be specified at clap level?
        let dest = destination.unwrap_or_else(|| String::from("~"));
        mv(&mut game, &dest);
    } else if let Some(Command::Reset {}) = opts.cmd {
        game.reset()
    } else if let Some(Command::Buy { item }) = opts.cmd {
        // when -s flag is provided, the positional argument is assumed to be an item
        shop(&mut game, &item);
    } else if let Some(Command::Use { item }) = opts.cmd {
        // when -i flag is provided, the positional argument is assumed to be an item
        use_item(&mut game, &item);
    } else if let Some(Command::Battle { run, bribe }) = opts.cmd {
        exit_code = battle(&mut game, run, bribe);
    } else {
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
        // FIXME this verbosity is annoying when using as a cd alias
        // it's not a good default. Re-enable based on a verbosity flag
        // log::short_status(&game);
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
    if let Some(mut enemy) = game.maybe_spawn_enemy() {
        if let Err(game::Error::GameOver) = game.maybe_battle(&mut enemy, run, bribe) {
            game.reset();
            exit_code = 1;
        }
    }
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
fn use_item(game: &mut Game, item_name: &Option<String>) {
    if let Some(item_name) = item_name {
        let item_name = sanitize(item_name);
        if let Err(game::Error::ItemNotFound) = game.use_item(&item_name) {
            println!("Item not found.");
        }
    } else {
        println!("{}", log::format_inventory(&game));
    }
}

// FIXME can this coercion be done as part of clap arg parsing?
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
