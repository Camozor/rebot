use std::{
    fs::{self, File},
    io::Write,
};

use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    model::player_stat::{UggLifetimeStats, UggRank},
    scraper::Scraper,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisteredPlayer {
    pub discord_id: u64,
    pub rematch_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayerWithStats {
    pub discord_id: u64,
    pub display_name: String,
    pub rank: Option<UggRank>,
    pub stats: UggLifetimeStats,
}

impl PlayerWithStats {
    pub fn get_all_matches(&self) -> i32 {
        self.stats.all.matches_played
    }

    pub fn get_wins(&self) -> i32 {
        self.stats.all.wins
    }

    pub fn get_loses(&self) -> i32 {
        self.get_all_matches() - self.get_wins()
    }

    pub fn get_win_rate(&self) -> String {
        let all_matches = self.get_all_matches() as f32;

        if all_matches == 0.0 {
            return format!("-");
        }
        let ratio = self.get_wins() as f32 / all_matches;
        let percent = ratio * 100.;

        format!("{:.1}", percent)
    }

    pub fn get_pretty_stats(&self) -> String {
        format!(
            "{}W {}L, {}% Win Rate",
            self.get_wins(),
            self.get_loses(),
            self.get_win_rate()
        )
    }

    pub fn pretty_rank(&self) -> String {
        if let None = self.rank {
            return format!("non classé");
        }

        let rank = self.rank.clone().unwrap();

        let league = rank.current_league;
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

        let division = 3 - rank.current_division;
        format!("{} {}", pretty_league, division)
    }

    pub fn estimate_hours_played(&self) -> i32 {
        let minutes = self.get_all_matches() * 6;
        minutes / 60
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerStore {
    #[serde(skip)]
    pub config: Config,

    pub registered_players: Vec<RegisteredPlayer>,
    pub players: Vec<PlayerWithStats>,
}

#[derive(Debug)]
pub enum RegisterError {
    WrongUrl(String),
}

#[derive(Debug)]
pub enum RefreshError {
    Err,
}

impl PlayerStore {
    pub fn new(config: &Config) -> Self {
        PlayerStore {
            config: config.clone(),
            registered_players: vec![],
            players: vec![],
        }
    }

    pub fn load_database(config: &Config) -> Self {
        let json_data = fs::read_to_string(&config.database_path);
        let json_data = match json_data {
            Ok(v) => v,
            Err(e) => {
                error!("Could not read {} database, {}", config.database_path, e);
                return PlayerStore::new(config);
            }
        };

        let store: PlayerStore = serde_json::from_str(&json_data).unwrap_or_else(|e| {
            error!("Could not parse {} database, {}", config.database_path, e);
            return PlayerStore::new(config);
        });

        PlayerStore {
            config: config.clone(),
            registered_players: store.registered_players,
            players: store.players,
        }
    }

    pub fn print(&self) {
        debug!("Registered players: {:?}", self.registered_players);
        debug!("Players: {:?}", self.players);
    }

    pub fn write_database(&self) {
        let json_data = serde_json::to_string_pretty(self);
        let json_data = match json_data {
            Ok(json_str) => json_str,
            Err(e) => {
                error!("Could not stringify store, {}", e);
                return;
            }
        };

        let file = File::create(&self.config.database_path);
        let mut file = match file {
            Ok(f) => f,
            Err(e) => {
                error!("Could not create {} file, {}", self.config.database_path, e);
                return;
            }
        };

        let written_bytes = file.write(json_data.as_bytes());
        if let Err(e) = written_bytes {
            error!("Could write to {} file, {}", self.config.database_path, e);
        }
    }

    pub fn register_player(
        &mut self,
        discord_id: u64,
        rematch_url: &str,
    ) -> Result<(), RegisterError> {
        if !rematch_url.starts_with("https://u.gg/rematch/profile/") {
            return Err(RegisterError::WrongUrl(String::from(
                "ton URL doit ressembler à https://u.gg/rematch/profile/steam/La%20m%C3%A9sange%20du%20Val%20d'Oise/76561198355389674",
            )));
        }

        let existing_player = self
            .registered_players
            .iter_mut()
            .find(|p| p.discord_id == discord_id);

        match existing_player {
            Some(player) => player.rematch_url = rematch_url.to_owned(),
            None => self.registered_players.push(RegisteredPlayer {
                discord_id,
                rematch_url: rematch_url.to_owned(),
            }),
        }

        Ok(())
    }

    pub async fn refresh_all(&mut self) -> Result<(), RefreshError> {
        let scraper = Scraper::new().await;
        let mut scraper = scraper.map_err(|_| RefreshError::Err)?;

        debug!("Scraper created");

        self.players = scraper.get_players_stats(&self.registered_players).await;
        self.write_database();

        Ok(())
    }

    pub fn get_player_stat(&self, discord_id: u64) -> Option<&PlayerWithStats> {
        self.players
            .iter()
            .find(|player| player.discord_id == discord_id)
    }

    pub fn get_all_players_stat(&self) -> Vec<PlayerWithStats> {
        self.players.clone()
    }

    pub fn find_try_harder(&self) -> Option<&PlayerWithStats> {
        self.players
            .iter()
            .max_by(|p1, p2| p1.get_all_matches().cmp(&p2.get_all_matches()))
    }
}
