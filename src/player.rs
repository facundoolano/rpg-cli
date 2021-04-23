pub struct Player {
    pub name: String,

    pub level: i32,
    pub xp: i32,

    pub max_hp: i32,
    pub current_hp: i32,

    pub strength: i32,
    pub speed: i32,
    pub luck: i32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            name: String::from("hero"),
            level: 1,
            xp: 0,
            max_hp: 20,
            current_hp: 20,
            strength: 10,
            speed: 5,
            luck: 3,
        }
    }
}
