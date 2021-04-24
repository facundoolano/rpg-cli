use crate::player::Player;
use serde::{Deserialize, Serialize};
use std::path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Game<'a> {
    pub player: Player,

    #[serde(borrow)]
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
