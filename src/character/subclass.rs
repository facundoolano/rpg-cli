use crate::character::class::Stat;

/// XP can be earned for one subclass at a time whichever one is selected as the default
/// Bonuses for each subclass can be applied together, for example you can be a level 10 hero with
/// a level 5 subclass warrior, level 2 wizard
///
/// TODO
/// - implement talents and skills for subclasses to further increase play diversity

#[derive(Serialize, Deserialize, Debug)]
pub enum SubclassType {
    Warrior,
    Apprentice,
    Cleric,
    Rogue,
}

impl SubclassType {
    pub fn get_subclass_name(subclass_type: SubclassType) -> &'static str {
        match subclass_type {
            SubclassType::Apprentice    => "Apprentice",
            SubclassType::Cleric        => "Cleric",
            SubclassType::Rogue         => "Rogue",
            SubclassType::Warrior       => "Warrior",
        }
    }

    pub fn get_subclass_names() -> Vec<&'static str> {
        let mut vec = Vec::new();
        vec.push(SubclassType::get_subclass_name(SubclassType::Warrior));
        vec.push(SubclassType::get_subclass_name(SubclassType::Apprentice));
        vec.push(SubclassType::get_subclass_name(SubclassType::Rogue));
        vec.push(SubclassType::get_subclass_name(SubclassType::Cleric));
        vec
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subclass {
    pub subclass_type: SubclassType,

    pub bonus_hp: Stat,
    pub bonus_strength: Stat,
    pub bonus_speed: Stat,
    pub bonus_mp: Stat,

    pub special_stat_name: &'static str,
    pub special_stat: Stat,

    pub level: i32,
    pub xp: i32,
}

impl Subclass {

    fn new(subclass_type: SubclassType, bonus_hp: Stat, bonus_strength: Stat,
           bonus_speed: Stat, bonus_mp: Stat, special_stat_name: &'static str,
           special_stat: Stat) -> Self {
        let mut subclass = Self {
            subclass_type,
            level: 1,
            xp: 0,
            bonus_hp,
            bonus_strength,
            bonus_speed,
            bonus_mp,
            special_stat_name,
            special_stat,
        };
        subclass
    }

    pub fn get_subclass_name(&self) -> &str {
        SubclassType::get_subclass_name(self.subclass_type)
    }

    /// Raise the level and all the character stats.
    fn increase_level(&mut self) {
        self.level += 1;
    }

    /// Add to the accumulated experience points, possibly increasing the level.
    pub fn add_experience(&mut self, xp: i32) -> i32 {
        self.xp += xp;

        let mut increased_levels = 0;
        let mut for_next = self.xp_for_next();
        while self.xp >= for_next {
            self.increase_level();
            self.xp -= for_next;
            increased_levels += 1;
            for_next = self.xp_for_next();
        }
        increased_levels
    }

    /// How many experience points are required to move to the next level.
    pub fn xp_for_next(&self) -> i32 {
        let exp = 1.7;
        let base_xp = 30.0;
        (base_xp * (self.level as f64).powf(exp)) as i32
    }

    /// Class options
    pub fn new_warrior() -> Subclass {
        let mut warrior = Subclass::new {
            subclass_type: SubclassType::Warrior,
            bonus_hp: Stat(0, 5),
            bonus_strength: Stat(0, 3),
            bonus_speed: Stat(0, 0),
            bonus_mp: Stat(0, 0),

            special_stat_name: "Endurance",
            special_stat: Stat(0, 1),
        };

        warrior
    }

    pub fn new_apprentice() -> Subclass {
        let mut apprentice = Subclass::new {
            subclass_type: SubclassType::Apprentice,
            bonus_hp: Stat(0, 1),
            bonus_strength: Stat(0, 0),
            bonus_speed: Stat(0, 0),
            bonus_mp: Stat(0, 4),

            special_stat_name: "Intelligence",
            special_stat: Stat(0, 1),
        };

        apprentice
    }
}

