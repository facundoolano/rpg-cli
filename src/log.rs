use crate::character::Character;
use crate::location::Location;
use colored::*;

pub fn enemy_appears(enemy: &Character, location: &Location) {
    println!("{}", character_at(&enemy, &location));
}

pub fn heal(player: &Character, location: &Location, recovered: i32) {
    if recovered > 0 {
        println!(
            "{} {}",
            character_at(&player, &location),
            format!("+{}hp", recovered).green()
        );
    }
}

pub fn attack(enemy: &Character, location: &Location, damage: i32) {
    let damage = if is_hero(&enemy) {
        format!("{}hp", -damage).bold().red()
    } else {
        format!("{}hp", -damage).white()
    };

    println!(
        "{} {}",
        character_at(&enemy, &location),
        damage
    );
}

pub fn battle_lost(player: &Character, location: &Location) {
    println!("{} \u{1F480}", character_at(&player, &location));
}

pub fn battle_won(player: &Character, location: &Location, xp: i32, level_up: bool, gold: i32) {
    let level_str = if level_up {
        " +level".cyan().to_string()
    } else {
        "".to_string()
    };

    println!(
        "{} {}{} {}",
        character_at(&player, &location),
        format!("+{}xp", xp).bold(),
        level_str,
        format!("+{}g", gold).yellow(),
    );
}

pub fn status(player: &Character, location: &Location) {
    println!("{}", character_at(&player, &location));
}

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = if is_hero(&self) {
            // FIXME use correct padding
            " hero".bold().to_string()
        } else {
            self.name.yellow().bold().to_string()
        };

        write!(f, "{}[{}]", name, self.level)
    }
}

// HELPERS

// NOTE: the pervasiveness of this function suggests that location should
// be an attribute of character, and that this function should be the
// implementation of the display trait
fn character_at(character: &Character, location: &Location) -> String {
    format!(
        "    {}{}{}@{}",
        character,
        hp_display(&character),
        xp_display(&character),
        location
    )
}

fn hp_display(character: &Character) -> String {
    bar_display(character.current_hp, character.max_hp, "green", "red")
}

fn xp_display(character: &Character) -> String {
    bar_display(
        character.xp,
        character.xp_for_next(),
        "cyan",
        // FIXME this one doesn't work
        "bright black",
    )
}

fn bar_display(current: i32, total: i32, current_color: &str, missing_color: &str) -> String {
    // FIXME this sometimes can still look unfilled at 100% or empty at >0%
    let units = (current as f64 * 4.0 / total as f64).ceil() as i32;
    let current = (0..units)
        .map(|_| "x")
        .collect::<String>()
        .color(current_color);
    let missing = (0..(4 - units))
        .map(|_| "-")
        .collect::<String>()
        .color(missing_color);
    format!("[{}{}]", current, missing)
}

fn is_hero(character: &Character) -> bool {
    // FIXME ugly hack, will fix some day --or not
    character.name == "hero"
}
