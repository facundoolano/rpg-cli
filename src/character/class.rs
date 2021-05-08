use rand::seq::IteratorRandom;

/// Character classes, which will determine the parameters to start and
/// increase the stats of the character.
#[derive(Debug, Clone)]
pub struct Class {
    pub name: &'static str,
    pub start_hp: i32,
    pub start_strength: i32,
    pub start_speed: i32,

    pub hp_rate: f64,
    pub strength_rate: f64,
    pub speed_rate: f64,
}

// pub enum Class {
//     Hero,
//     Test,

//     Rat,
//     Wolf,
//     Snake,
//     Slime,
//     Spider,

//     Zombie,
//     Orc,
//     Skeleton,
//     Demon,
//     Vampire,
//     Dragon,
//     Golem,

//     Chimera,
//     Basilisk,
//     Minotaur,
//     Balrog,
//     Phoenix,
// }

// TODO review stats
const RAT: Class = Class {
    name: "rat",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const WOLF: Class = Class {
    name: "wolf",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};



impl Class {
    pub const HERO: Self = Self {
        name: "hero",
        start_hp: 25,
        start_strength: 10,
        start_speed: 5,

        hp_rate: 0.3,
        strength_rate: 0.2,
        speed_rate: 0.1,
    };

    // FIXME add the rest
    const NEAR_ENEMIES: &'static [Self] = &[RAT, WOLF];
    const MEDIUM_ENEMIES: &'static [Self] = &[RAT, WOLF];
    const FAR_ENEMIES: &'static [Self] = &[RAT, WOLF];

    pub fn random_enemy(distance_from_home: i32) -> &'static Self {
        match distance_from_home {
            n if n <= 4  => Self::random_choice(Self::NEAR_ENEMIES),
            n if n <= 9  => Self::random_choice(Self::MEDIUM_ENEMIES),
            _ => Self::random_choice(Self::FAR_ENEMIES)
        }
    }

    fn random_choice(options: &[Self]) -> &Self {
        let mut rng = rand::thread_rng();
        options.iter().choose(&mut rng).unwrap()
    }
}
