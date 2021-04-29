use game::Game;

mod character;
mod game;
mod location;

use crate::location::Location;

fn main() {
    println!();
    let mut game = Game::load().unwrap_or_else(|_| Game::new());

    if let Some(dest) = std::env::args().nth(1) {
        let dest = Location::from(&dest);

        match game.walk_towards(&dest) {
            Err(game::Error::GameOver) => game.reset(),
            _ => game.save().unwrap(),
        }
    }

    // FIXME this prints redundat information when there's another event before
    // but taking it out makes no-event movements weird
    // perhaps that gets solved by using the chdir side effect?
    // alternatively, the event log printing could have the smarts
    // to print status if there are no other printable events
    println!("{}", game.player.display_at(&game.location));
    println!();
}
