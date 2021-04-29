use crate::character::Character;
use crate::location::Location;
use colored::*;

pub fn enemy_appears(enemy: &Character, location: &Location) {
    log(&enemy, &location, "");
}

pub fn heal(player: &Character, location: &Location, recovered: i32) {
    if recovered > 0 {
        log(
            &player,
            &location,
            &format!("+{}hp", recovered).green().to_string(),
        );
    }
}

pub fn attack(enemy: &Character, location: &Location, damage: i32) {
    let damage = if is_hero(&enemy) {
        format!("{}hp", -damage).bold().red()
    } else {
        format!("{}hp", -damage).white()
    };

    log(&enemy, &location, &damage.to_string());
}

pub fn battle_lost(player: &Character, location: &Location) {
    log(&player, &location, "\u{1F480}");
}

pub fn battle_won(player: &Character, location: &Location, xp: i32, level_up: bool, gold: i32) {
    let level_str = if level_up {
        " +level".cyan().to_string()
    } else {
        "".to_string()
    };

    log(
        &player,
        &location,
        &format!(
            "{}{} {}",
            format!("+{}xp", xp).bold(),
            level_str,
            format!("+{}g", gold).yellow()
        ),
    );
}

pub fn status(player: &Character, location: &Location) {
    log(&player, &location, "");
}

// HELPERS

/// Generic log function. At the moment all output of the game is structured as
/// of a player status at some location, with an optional event suffix.
fn log(character: &Character, location: &Location, suffix: &str) {
    println!(
        "    {}[{}]{}{}@{} {}",
        name(&character),
        character.level,
        hp_display(&character),
        xp_display(&character),
        location,
        suffix
    );
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

fn name(character: &Character) -> String {
    if is_hero(&character) {
        // FIXME use correct padding
        " hero".bold().to_string()
    } else {
        character.name.yellow().bold().to_string()
    }
}

fn is_hero(character: &Character) -> bool {
    // FIXME ugly hack, will fix some day --or not
    character.name == "hero"
}
