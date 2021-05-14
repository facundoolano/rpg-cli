use std::collections::HashMap;
use std::fmt::Display;

use super::Equipment;
use crate::character::Character;
use crate::game::Game;

pub fn list(player: &Character) {
    for item in available_items(player).values() {
        println!("{}  {}g", item, item.cost());
    }
}

// FIXME try to remove duplication

pub fn buy(game: &mut Game, item: &str) -> Result<(), String> {
    let player = &mut game.player;
    let mut items = available_items(player);
    if let Some(item) = items.remove(&item.to_lowercase()) {
        item.buy(game)?;
        // FIXME do something with it
        // TODO will require differentiating sword and shield
        // game.player.sword = Some(sword);
        Ok(())
    } else {
        Err("item not available".to_string())
    }
}

fn available_items(player: &Character) -> HashMap<String, Box<dyn Shoppable>> {
    let mut items = HashMap::<String, Box<dyn Shoppable>>::new();

    if let Some(sword) = next_equipment(&player, "sword", &player.sword) {
        items.insert("sword".to_string(), Box::new(sword));
    }

    if let Some(shield) = next_equipment(&player, "shield", &player.shield) {
        items.insert("shield".to_string(), Box::new(shield));
    }

    let potion = super::Potion::new(available_level(&player));
    items.insert("potion".to_string(), Box::new(potion));

    let escape = super::Escape::new();
    items.insert("escape".to_string(), Box::new(escape));

    items
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

trait Shoppable: Display {
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
