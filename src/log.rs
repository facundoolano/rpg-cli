use crate::character::Character;
use crate::location::Location;
use colored::*;

pub fn enemy_appears(enemy: &Character, location: &Location) {
    // TODO display_at function should go in this module
    println!("{}", enemy.display_at(&location));
}

pub fn heal(player: &Character, location: &Location, recovered: i32) {
    if recovered > 0 {
        println!(
            "{} {}",
            player.display_at(&location),
            format!("+{}hp", recovered).green()
        );
    }
}

pub fn attack(enemy: &Character, location: &Location, damage: i32) {
    println!(
        "{} {}",
        enemy.display_at(&location),
        format!("{}hp", -damage)
    );
}

pub fn battle_lost(player: &Character, location: &Location) {
    println!("{} \u{1F480}", player.display_at(&location));
}

pub fn battle_won(player: &Character, location: &Location, xp: i32, level_up: bool, gold: i32) {
    let level_str = if level_up {
        " +level".cyan().to_string()
    } else {
        "".to_string()
    };

    println!(
        "{} {}{} {}",
        player.display_at(&location),
        format!("+{}xp", xp).bold(),
        level_str,
        format!("+{}g", gold).yellow(),
    );
}

pub fn status(player: &Character, location: &Location) {
    println!("{}", player.display_at(&location));
}
