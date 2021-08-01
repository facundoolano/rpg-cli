use crate::character;
use crate::game;
use crate::game::Game;
use crate::item;
use crate::location::Location;
use crate::log;

/// Attempt to move the hero to the supplied location, possibly engaging
/// in combat along the way.
pub fn change_dir(game: &mut Game, dest: &str, run: bool, bribe: bool, force: bool) -> i32 {
    if let Ok(dest) = Location::from(&dest) {
        if force {
            game.location = dest;
        } else if let Err(character::Dead) = game.go_to(&dest, run, bribe) {
            game.reset();
            return 1;
        }
    } else {
        println!("No such file or directory");
        return 1;
    }
    0
}

/// Potentially run a battle at the current location, independently from
/// the hero's movement.
pub fn battle(game: &mut Game, run: bool, bribe: bool) -> i32 {
    let mut exit_code = 0;
    if let Some(mut enemy) = game.maybe_spawn_enemy() {
        if let Err(character::Dead) = game.maybe_battle(&mut enemy, run, bribe) {
            game.reset();
            exit_code = 1;
        }
    }
    exit_code
}

/// Set the class for the player character
pub fn class(game: &mut Game, class_name: &Option<String>) {
    if let Some(class_name) = class_name {
        let class_name = sanitize(class_name);
        match game.change_class(&class_name) {
            Err(game::ClassChangeError::NotAtHome) => {
                println!("Class change is only allowed at home.");
            }
            Err(game::ClassChangeError::NotFound) => {
                println!("Unknown class name.")
            }
            Ok(()) => {}
        }
    } else {
        let player_classes: Vec<String> =
            character::class::Class::names(character::class::Category::Player)
                .iter()
                .cloned()
                .collect();
        println!("Options: {}", player_classes.join(", "));
    }
}

/// Buy an item from the shop or list the available items if no item name is provided.
/// Shopping is only allowed when the player is at the home directory.
pub fn shop(game: &mut Game, item_name: &Option<String>) {
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
        // FIXME this rule shouldn't be enforced here
        println!("Shop is only allowed at home.")
    }
}

/// Use an item from the inventory or list the inventory contents if no item name is provided.
pub fn use_item(game: &mut Game, item_name: &Option<String>) {
    if let Some(item_name) = item_name {
        let item_name = sanitize(item_name);
        if let Err(game::ItemNotFound) = game.use_item(&item_name) {
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
        "e" | "ether" => "ether",
        "es" | "escape" => "escape",
        "sw" | "sword" => "sword",
        "sh" | "shield" => "shield",
        n => n,
    };
    name.to_string()
}
