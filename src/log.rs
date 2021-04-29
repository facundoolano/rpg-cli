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
    println!(
        "{} {}",
        character_at(&enemy, &location),
        format!("{}hp", -damage)
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
        // FIXME ugly
        let name = if self.name == "hero" {
            // FIXME use correct padding
            " hero".bold().to_string()
        } else {
            self.name.yellow().bold().to_string()
        };

        write!(f, "{}[{}]", name, self.level)
    }
}

// HELPERS

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
    // FIXME this sometimes can still look unfilled at 100%
    let current_units = (character.current_hp as f64 * 4.0 / character.max_hp as f64).ceil() as i32;
    let green = (0..current_units).map(|_| "x").collect::<String>().green();
    let red = (0..(4 - current_units))
        .map(|_| "-")
        .collect::<String>()
        .red();
    format!("[{}{}]", green, red)
}

// FIXME duplicated bar display
fn xp_display(character: &Character) -> String {
    let current_units = character.xp * 4 / character.xp_for_next();
    let green = (0..current_units).map(|_| "x").collect::<String>().cyan();
    let red = (0..(4 - current_units))
        .map(|_| "-")
        .collect::<String>()
        .bright_black();
    format!("[{}{}]", green, red)
}
