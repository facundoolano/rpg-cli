use super::Equipment;
use crate::character::Character;

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

pub fn buy(item: &str) {
    println!("There isn't any {} for sale right now.", item);
    todo!();
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
