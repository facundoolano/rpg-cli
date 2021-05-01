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
pub fn player_attack(enemy: &Character, location: &Location, damage: i32) {
    log(
        &enemy,
        &location,
        &format!("{}hp", -damage).white().to_string(),
    );
}

pub fn enemy_attack(player: &Character, location: &Location, damage: i32) {
    log(
        &player,
        &location,
        &format!("{}hp", -damage).bold().red().to_string(),
    );
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
        "bright black",
    )
}

fn bar_display(current: i32, total: i32, current_color: &str, missing_color: &str) -> String {
    let (filled, rest) = bar_slots(4, total, current);
    let current = (0..filled)
        .map(|_| "x")
        .collect::<String>()
        .color(current_color);
    let missing = (0..rest)
        .map(|_| "-")
        .collect::<String>()
        .color(missing_color);
    format!("[{}{}]", current, missing)
}

fn bar_slots(slots: i32, total: i32, current: i32) -> (i32, i32) {
    let units = (current as f64 * slots as f64 / total as f64).ceil() as i32;
    (units, slots - units)
}

fn name(character: &Character) -> String {
    // FIXME ugly hack, will fix some day --or not
    if character.name == "hero" {
        // FIXME use correct padding
        " hero".bold().to_string()
    } else {
        character.name.yellow().bold().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_slots() {
        // simple case 1:1 between points and slots
        let slots = 4;
        let total = 4;
        assert_eq!((0, 4), bar_slots(slots, total, 0));
        assert_eq!((1, 3), bar_slots(slots, total, 1));
        assert_eq!((2, 2), bar_slots(slots, total, 2));
        assert_eq!((3, 1), bar_slots(slots, total, 3));
        assert_eq!((4, 0), bar_slots(slots, total, 4));

        let total = 10;
        assert_eq!((0, 4), bar_slots(slots, total, 0));
        assert_eq!((1, 3), bar_slots(slots, total, 1));
        assert_eq!((1, 3), bar_slots(slots, total, 2));
        assert_eq!((2, 2), bar_slots(slots, total, 3));
        assert_eq!((2, 2), bar_slots(slots, total, 4));
        assert_eq!((2, 2), bar_slots(slots, total, 5));
        assert_eq!((3, 1), bar_slots(slots, total, 6));
        assert_eq!((3, 1), bar_slots(slots, total, 7));
        // this one I would maybe like to show as 3, 1
        assert_eq!((4, 0), bar_slots(slots, total, 8));
        assert_eq!((4, 0), bar_slots(slots, total, 9));
        assert_eq!((4, 0), bar_slots(slots, total, 10));
    }

}
