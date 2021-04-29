use game::Game;

mod character;
mod game;
mod location;
mod log;

use crate::location::Location;

fn main() {
    let mut game = Game::load().unwrap_or_else(|_| Game::new());

    if let Some(dest) = std::env::args().nth(1) {
        let dest = Location::from(&dest);

        match game.walk_towards(&dest) {
            Err(game::Error::GameOver) => game.reset(),
            _ => game.save().unwrap(),
        }
    } else {
        log::status(&game.player, &game.location);
    }
}
