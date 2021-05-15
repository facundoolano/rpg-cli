use game::Game;

mod character;
mod game;
mod item;
mod location;
mod log;
mod randomizer;

use crate::location::Location;
use clap::Clap;

#[derive(Clap)]
struct Opts {
    /// Moves the hero to the supplied destination.
    /// When omitted to just prints the hero's status
    destination: Option<String>,

    /// Prints the hero's current location
    #[clap(long)]
    pwd: bool,

    /// Resets the current game.
    #[clap(long)]
    reset: bool,

    /// Buys an item from the shop.
    /// If name is omitted lists the items available for sale.
    #[clap(short, long)]
    shop: bool,

    /// Uses an item from the inventory.
    /// If name is omitted lists the inventory contents.
    #[clap(short, long)]
    inventory: bool,
}

fn main() {
    let opts: Opts = Opts::parse();

    let mut game = Game::load().unwrap_or_else(|_| Game::new());

    if opts.pwd {
        println!("{}", game.location.path_string());
    } else if opts.reset {
        game.reset()
    } else if opts.shop {
        // when -s flag is provided, the positional argument is assumed to be an item
        shop(&mut game, &opts.destination);
    } else if opts.inventory {
        // when -i flag is provided, the positional argument is assumed to be an item
        inventory(&mut game, &opts.destination);
    } else if let Some(dest) = opts.destination {
        go_to(&mut game, &dest);
    } else {
        log::status(&game);
    }

    game.save().unwrap()
}

/// Main command, attempt to move the hero to the supplied location,
/// possibly engaging in combat along the way.
fn go_to(game: &mut Game, dest: &str) {
    if let Ok(dest) = Location::from(&dest) {
        if let Err(game::Error::GameOver) = game.go_to(&dest) {
            game.reset();
        }
    } else {
        println!("No such file or directory");
        std::process::exit(1);
    }
}

/// Placeholder, for now there's no support for items.
fn shop(game: &mut Game, item: &Option<String>) {
    if game.location.is_home() {
        if let Some(item) = item {
            // FIXME print error
            item::shop::buy(game, item).unwrap();
        } else {
            item::shop::list(&game.player);
        }
    } else {
        println!("Shop is only allowed at home.")
    }
}

/// Placeholder, for now there's no support for items.
fn inventory(game: &mut Game, item_name: &Option<String>) {
    if let Some(item_name) = item_name {
        // TODO handle missing error
        game.use_item(item_name);
    } else {
        println!("item:{{{}}}", game.format_inventory());
    }
}
