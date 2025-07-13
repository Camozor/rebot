use crate::{model::player_stat::Rank, scraper::Scraper};

#[derive(Debug)]
pub struct RegisteredPlayer {
    pub discord_id: u64,
    pub rematch_url: String,
}

#[derive(Debug)]
pub struct PlayerWithStats {
    pub discord_id: u64,
    pub display_name: String,
    pub rank: Rank,
}

#[derive(Debug)]
pub struct PlayerStore {
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
    pub fn new() -> Self {
        PlayerStore {
            registered_players: vec![],
            players: vec![],
        }
    }

    pub fn with_data() -> Self {
        PlayerStore {
            registered_players: vec![RegisteredPlayer {
                discord_id: 428258972156559362,
                rematch_url: String::from(
                    "https://u.gg/rematch/profile/steam/La%20m%C3%A9sange%20du%20Val%20d'Oise/76561198355389674",
                ),
            }],
            players: vec![],
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
        let mut scraper = Scraper::new().await.map_err(|_| RefreshError::Err)?;

        let result = scraper.get_players_stats(&self.registered_players).await;
        self.players = result;

        Ok(())
    }

    pub fn get_player_stat(&self, discord_id: u64) -> Option<&PlayerWithStats> {
        self.players
            .iter()
            .find(|player| player.discord_id == discord_id)
    }
}
