use rand::Rng;

pub struct Randomizer {}

#[cfg(not(test))]
impl Randomizer {
    pub fn enemy_delta() -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(-1..2)
    }

    pub fn should_enemy_appear() -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_ratio(1, 3)
    }
}

#[cfg(test)]
impl Randomizer {
    pub fn enemy_delta() -> i32 {
        0
    }

    pub fn should_enemy_appear() -> bool {
        true
    }
}
