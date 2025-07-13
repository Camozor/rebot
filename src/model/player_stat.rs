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
