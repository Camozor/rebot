use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PlayerStat {
    pub player: Player,
    pub rank: Rank,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub display_name: String,
    pub level: i32,
}

#[derive(Debug, Deserialize)]
pub struct Rank {
    pub current_league: i32,
    pub current_division: i32,
}

impl Rank {
    pub fn pretty_rank(&self) -> String {
        let league = self.current_league;
        let pretty_league = match league {
            0 => String::from("Bronze"),
            1 => String::from("Silver"),
            2 => String::from("Gold"),
            3 => String::from("Platinum"),
            4 => String::from("Diamond"),
            5 => String::from("Elite"),
            _ => String::from("Unknown"),
        };

        format!("{} {}", pretty_league, self.current_division)
    }
}
