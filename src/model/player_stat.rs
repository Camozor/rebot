use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UggPlayerStat {
    pub player: UggPlayer,
    pub rank: Option<UggRank>,
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
