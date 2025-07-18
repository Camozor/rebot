use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize)]
pub struct Rank {
    pub current_league: i32,
    pub current_division: i32,
}

impl Rank {
    pub fn pretty_rank(&self) -> String {
        let league = self.current_league;
        let pretty_league = match league {
            0 => String::from("bronze"),
            1 => String::from("argent"),
            2 => String::from("or"),
            3 => String::from("platine"),
            4 => String::from("diamant"),
            5 => String::from("élite"),
            _ => String::from("inconnu"),
        };

        let division = 3 - self.current_division;
        format!("{} {}", pretty_league, division)
    }
}
