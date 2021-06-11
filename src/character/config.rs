use crate::character::class::{FAR_ENEMIES, MEDIUM_ENEMIES, NEAR_ENEMIES};
use crate::character::Class;
use crate::location::Distance;
use serde::{Deserialize, Serialize};
use std::{fs, path};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CharacterConfig {
    pub hero: Class,
    pub enemies: Vec<Class>,
}

impl CharacterConfig {
    pub fn load(file: path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let f = fs::File::open(file)?;
        let config: CharacterConfig = serde_yaml::from_reader(f)?;
        config.init();
        Ok(config)
    }

    fn enemies(self) -> Vec<Class> {
        self.enemies
    }

    fn init(&self) {
        FAR_ENEMIES.set(self.clone().far_enemies()).unwrap(); // TODO: avoid clone
        MEDIUM_ENEMIES.set(self.clone().mid_enemies()).unwrap(); // TODO: avoid clone
        NEAR_ENEMIES.set(self.clone().near_enemies()).unwrap(); // TODO: avoid clone
    }

    fn near_enemies(self) -> Vec<Class> {
        self.enemies()
            .into_iter()
            .filter(|x| matches!(x.distance, Some(Distance::Near(_))))
            .collect()
    }
    fn mid_enemies(self) -> Vec<Class> {
        self.enemies()
            .into_iter()
            .filter(|x| matches!(x.distance, Some(Distance::Mid(_))))
            .collect()
    }
    fn far_enemies(self) -> Vec<Class> {
        self.enemies
            .into_iter()
            .filter(|x| matches!(x.distance, Some(Distance::Far(_))))
            .collect()
    }
}
