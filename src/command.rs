use crate::character;
use crate::game;
use crate::game::Game;
use crate::item;
use crate::location::Location;
use crate::log;

use clap::Clap;

#[derive(Clap)]
pub enum Command {
    /// Display the hero's status [default]
    #[clap(aliases=&["s", "status"], display_order=0)]
    Stat,

    /// Moves the hero to the supplied destination, potentially initiating battles along the way.
    #[clap(name = "cd", display_order = 1)]
    ChangeDir {
        /// Directory to move to.
        #[clap(default_value = "~")]
        destination: String,

        /// Attempt to avoid battles by running away.
        #[clap(long)]
        run: bool,

        /// Attempt to avoid battles by bribing the enemy.
        #[clap(long)]
        bribe: bool,

        /// Move the hero's to a different location without spawning enemies.
        /// Intended for scripts and shell integration.
        #[clap(short, long)]
        force: bool,
    },

    /// Inspect the directory contents, possibly finding treasure chests and hero tombstones.
    #[clap(name = "ls", display_order = 1)]
    Inspect,

    /// Buys an item from the shop.
    /// If name is omitted lists the items available for sale.
    #[clap(alias = "b", display_order = 2)]
    Buy { items: Vec<String> },

    /// Uses an item from the inventory.
    #[clap(alias = "u", display_order = 3)]
    Use { items: Vec<String> },

    /// Prints the quest todo list.
    #[clap(alias = "t", display_order = 4)]
    Todo,

    /// Resets the current game.
    Reset {
        /// Reset data files, losing cross-hero progress.
        #[clap(long)]
        hard: bool,
    },

    /// Change the character class.
    /// If name is omitted lists the available character classes.
    Class { name: Option<String> },

    /// Prints the hero's current location
    #[clap(name = "pwd")]
    PrintWorkDir,

    /// Potentially initiates a battle in the hero's current location.
    Battle {
        /// Attempt to avoid battles by running away.
        #[clap(long)]
        run: bool,

        /// Attempt to avoid battles by bribing the enemy.
        #[clap(long)]
        bribe: bool,
    },
}

pub fn run(cmd: Option<Command>, game: &mut Game) -> i32 {
    let mut exit_code = 0;

    match cmd.unwrap_or(Command::Stat) {
        Command::Stat => log::status(&game),
        Command::ChangeDir {
            destination,
            run,
            bribe,
            force,
        } => {
            exit_code = change_dir(game, &destination, run, bribe, force);
        }
        Command::Inspect => {
            game.inspect();
        }
        Command::Class { name } => class(game, &name),
        Command::Battle { run, bribe } => {
            exit_code = battle(game, run, bribe);
        }
        Command::PrintWorkDir => println!("{}", game.location.path_string()),
        Command::Reset { .. } => game.reset(),
        Command::Buy { items } => exit_code = shop(game, &items),
        Command::Use { items } => exit_code = use_item(game, &items),
        Command::Todo => {
            let (todo, done) = game.quests.list(&game);
            log::quest_list(&todo, &done);
        }
    }

    exit_code
}

/// Attempt to move the hero to the supplied location, possibly engaging
/// in combat along the way.
fn change_dir(game: &mut Game, dest: &str, run: bool, bribe: bool, force: bool) -> i32 {
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
fn battle(game: &mut Game, run: bool, bribe: bool) -> i32 {
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
fn class(game: &mut Game, class_name: &Option<String>) {
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
fn shop(game: &mut Game, items: &[String]) -> i32 {
    if game.location.is_home() {
        if items.is_empty() {
            item::shop::list(game);
            0
        } else {
            for item_name in items {
                let item_name = sanitize(item_name);
                match item::shop::buy(game, &item_name) {
                    Err(item::shop::Error::NotEnoughGold) => {
                        println!("Not enough gold.");
                        return 1;
                    }
                    Err(item::shop::Error::ItemNotAvailable) => {
                        println!("Item not available.");
                        return 1;
                    }
                    Ok(()) => {}
                }
            }
            0
        }
    } else {
        // FIXME this rule shouldn't be enforced here
        println!("Shop is only allowed at home.");
        1
    }
}

/// Use an item from the inventory or list the inventory contents if no item name is provided.
fn use_item(game: &mut Game, items: &[String]) -> i32 {
    if items.is_empty() {
        println!("{}", log::format_inventory(&game));
    } else {
        for item_name in items {
            let item_name = sanitize(item_name);
            if let Err(game::ItemNotFound) = game.use_item(&item_name) {
                println!("Item not found.");
                return 1;
            }
        }
    }
    0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_dir_battle() {
        let mut game = Game::new();
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: false,
        };

        // increase level to ensure win
        for _ in 0..10 {
            game.player.add_experience(game.player.xp_for_next());
        }

        let result = run(Some(cmd), &mut game);

        assert_eq!(0, result);
        assert!(game.player.xp > 0);
        assert!(game.gold > 0);
    }

    #[test]
    fn change_dir_dead() {
        let mut game = Game::new();
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: false,
        };

        // reduce stats to ensure loss
        game.player.speed = 1;
        game.player.strength = 1;
        game.player.current_hp = 1;

        game.gold = 100;
        game.player.xp = 100;

        let result = run(Some(cmd), &mut game);

        assert_eq!(1, result);
        // game reset
        assert_eq!(game.player.max_hp, game.player.current_hp);
        assert_eq!(0, game.gold);
        assert_eq!(0, game.player.xp);
    }

    #[test]
    fn change_dir_home() {
        let mut game = Game::new();

        assert!(game.location.is_home());

        // force move to a non home location
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: true,
        };

        let result = run(Some(cmd), &mut game);
        assert_eq!(0, result);
        assert!(!game.location.is_home());

        game.player.current_hp = 1;

        // back home (without forcing)
        let cmd = Command::ChangeDir {
            destination: "~".to_string(),
            run: false,
            bribe: false,
            force: false,
        };

        let result = run(Some(cmd), &mut game);
        assert_eq!(0, result);
        assert!(game.location.is_home());
        assert_eq!(game.player.max_hp, game.player.current_hp);
    }

    #[test]
    fn inspect_tombstone() {
        // die at non home with some gold
        let mut game = Game::new();
        assert!(game.tombstones.is_empty());

        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: false,
        };

        // reduce stats to ensure loss
        game.player.speed = 1;
        game.player.strength = 1;
        game.player.current_hp = 1;

        game.gold = 100;
        run(Some(cmd), &mut game);

        assert_eq!(0, game.gold);
        assert!(!game.tombstones.is_empty());

        // force move to the previous dead location
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: true,
        };
        run(Some(cmd), &mut game);

        // inspect to pick up lost gold
        let cmd = Command::Inspect;
        let result = run(Some(cmd), &mut game);
        assert_eq!(0, result);
        assert!(game.tombstones.is_empty());

        // includes +200g for visit tombstone quest
        assert_eq!(300, game.gold);
    }

    #[test]
    fn buy_use_item() {
        let mut game = Game::new();
        assert!(game.inventory().is_empty());

        // not buy if not enough money
        let cmd = Command::Buy {
            item: Some(String::from("potion")),
        };
        let result = run(Some(cmd), &mut game);
        assert_eq!(1, result);
        assert!(game.inventory().is_empty());

        // buy potion
        game.gold = 200;
        let cmd = Command::Buy {
            item: Some(String::from("potion")),
        };
        let result = run(Some(cmd), &mut game);
        assert_eq!(0, result);
        assert!(!game.inventory().is_empty());
        assert_eq!(0, game.gold);

        // use potion
        game.player.current_hp -= 1;
        let cmd = Command::Use {
            item: Some(String::from("potion")),
        };
        let result = run(Some(cmd), &mut game);
        assert_eq!(0, result);
        assert!(game.inventory().is_empty());
        assert_eq!(game.player.max_hp, game.player.current_hp);

        // not buy if not home
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: true,
        };
        run(Some(cmd), &mut game);

        game.gold = 200;
        let cmd = Command::Buy {
            item: Some(String::from("potion")),
        };
        let result = run(Some(cmd), &mut game);
        assert_eq!(1, result);
        assert!(game.inventory().is_empty());
    }
}
