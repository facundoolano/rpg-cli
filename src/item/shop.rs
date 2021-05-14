use super::Equipment;
use crate::character::Character;
use crate::game::Game;

// FIXME try to remove duplication

pub fn list(player: &Character) {
    if let Some(sword) = next_equipment(&player, "sword", &player.sword) {
        println!("{}  {}g", sword, sword.cost());
    }

    if let Some(shield) = next_equipment(&player, "shield", &player.shield) {
        println!("{}  {}g", shield, shield.cost());
    }

    let potion = super::Potion::new(available_level(&player));
    println!("{} {}g", potion, potion.cost());

    let escape = super::Escape::new();
    println!("{} {}g", escape, escape.cost());
}

// FIXME try to remove duplication

pub fn buy(game: &mut Game, item: &str) -> Result<(), String> {
    let player = &mut game.player;

    match item.to_lowercase().as_str() {
        "sw" | "sword" => {
            if let Some(sword) = next_equipment(&player, "sword", &player.sword) {
                sword.buy(game)?;
                game.player.sword = Some(sword);
            } else {
                return Err("item not available".to_string());
            }
        }
        "sh" | "shield" => {
            if let Some(shield) = next_equipment(&player, "shield", &player.shield) {
                shield.buy(game)?;
                game.player.shield = Some(shield);
            } else {
                return Err("item not available".to_string());
            }
        }
        "p" | "potion" => {
            let potion = super::Potion::new(available_level(&player));
            potion.buy(game)?;
            // FIXME add to inventory
        }
        "e" | "escape" => {
            let escape = super::Escape::new();
            escape.buy(game)?;
            // FIXME add to inventory
        }
        _ => {
            return Err("item not available".to_string());
        }
    }
    Ok(())
}

fn next_equipment(
    player: &Character,
    name: &str,
    current: &Option<Equipment>,
) -> Option<Equipment> {
    let level = available_level(&player);
    if let Some(sword) = &current {
        if sword.level >= level {
            return None;
        }
    }
    Some(Equipment::new(name, level))
}

/// The offered items/equipment have levels e.g. potion[1], sword[5], etc.
/// they become available for purchase only when the player reaches that level
fn available_level(player: &Character) -> i32 {
    // allow level 1 or level 5n
    std::cmp::max(1, (player.level / 5) * 5)
}

trait Shoppable {
    fn cost(&self) -> i32;
    fn buy(&self, game: &mut Game) -> Result<(), String> {
        if game.gold < self.cost() {
            return Err("Not enough gold".to_string());
        }
        game.gold -= self.cost();
        Ok(())
    }
}

impl Shoppable for Equipment {
    fn cost(&self) -> i32 {
        self.level * 500
    }
}

impl Shoppable for super::Potion {
    fn cost(&self) -> i32 {
        self.level * 200
    }
}

impl Shoppable for super::Escape {
    fn cost(&self) -> i32 {
        1000
    }
}
