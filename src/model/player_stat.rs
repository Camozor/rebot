use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UggPlayerStat {
    pub player: UggPlayer,
    pub rank: UggRank,
    pub lifetime_stats: UggLifetimeStats,
}

#[derive(Debug, Deserialize)]
pub struct UggPlayer {
    pub display_name: String,
    pub level: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UggRank {
    pub current_league: i32,
    pub current_division: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UggLifetimeStats {
    #[serde(rename = "All")]
    pub all: UggAllLifetimeStats,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UggAllLifetimeStats {
    pub matches_played: i32,
    pub wins: i32,
}

impl UggRank {
    pub fn pretty_rank(&self) -> String {
        let league = self.current_league;
        let pretty_league = match league {
            0 => String::from("bronze"),
            1 => String::from("argent"),
            2 => String::from("or"),
            3 => String::from("platine"),
            4 => String::from("diamant"),
            5 => String::from("maître"),
            6 => String::from("élite"),
            _ => String::from("inconnu"),
        };

        if league == 6 {
            return format!("{}", pretty_league);
        }

        let division = 3 - self.current_division;
        format!("{} {}", pretty_league, division)
    }
}
