use game::Game;

mod character;
mod game;
mod location;

use crate::location::Location;

fn main() {
    let mut game = match Game::load() {
        Ok(game) => game,
        Err(_) => {
            let game = Game::new();
            game.save().unwrap();
            game
        }
    };

    if let Some(dest) = std::env::args().nth(1) {
        let dest = Location::from(&dest);

        match game.walk_towards(&dest) {
            Err(game::Error::GameOver) => game.reset().unwrap(),
            _ => game.save().unwrap(),
        }
    }

    player_status(&game);
}

fn player_status(game: &Game) {
    let player = &game.player;
    println!("{}@{} ", player.name, game.location);
    println!(
        "lv:{} hp:{}/{} xp:{}",
        player.level, player.current_hp, player.max_hp, player.xp
    );
}
