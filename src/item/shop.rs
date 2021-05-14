use crate::character::Character;

pub fn list(player: &Character) {
    if let Some(sword) = next_equipment(&player, &player.sword) {
        println!("sword[{}]  {}g", sword.level, sword.cost());
    }

    if let Some(shield) = next_equipment(&player, &player.shield) {
        println!("shield[{}]  {}g", shield.level, shield.cost());
    }

    let escape = super::Escape::new();
    println!("escape {}g", escape.cost());

    let potion = super::Potion::new(available_level(&player));
    println!("potion[{}] {}g", potion.level, potion.cost());
}

pub fn buy(item: &str) {
    println!("There isn't any {} for sale right now.", item);
    todo!();
}

fn next_equipment(player: &Character, equip: &Option<super::Equipment>) -> Option<super::Equipment> {
    let level = available_level(&player);
    if let Some(sword) = &equip {
        if sword.level >= level {
            return None;
        }
    }
    Some(super::Equipment::new(level))
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

impl Shoppable for super::Equipment {
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
