use crate::player::Player;
use std::path;

pub struct Game<'a> {
    pub player: Player,
    pub location: &'a path::Path,
}

impl Game<'_> {
    pub fn new() -> Self {
        Self {
            location: path::Path::new("~"),
            player: Player::new(),
        }
    }
}
