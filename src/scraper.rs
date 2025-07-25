use chromiumoxide::cdp::browser_protocol::network::{
    EnableParams as NetworkEnableParams, EventRequestWillBeSent,
};
use chromiumoxide::{
    Browser, BrowserConfig, browser::HeadlessMode,
    cdp::browser_protocol::network::EventLoadingFinished,
};
use tokio::time::{Duration, sleep, timeout};

use chromiumoxide::cdp::browser_protocol::network::GetResponseBodyParams;
use futures::StreamExt;
use log::{debug, error, info};

use crate::model::player_stat::PlayerStat;
use crate::player_store::{PlayerWithStats, RegisteredPlayer};

pub struct Scraper {
    browser: Browser,
    pub handler_task: tokio::task::JoinHandle<()>,
}

#[derive(Debug)]
pub enum ScrapeError {
    PageInit(String),
    RequestNotFound,
    RequestContent,
    Timeout,
}

#[derive(Debug)]
pub enum ScraperInitError {
    Browser(String),
}

impl Scraper {
    pub async fn new() -> Result<Self, ScraperInitError> {
        info!("Browser creation");
        let config = BrowserConfig::builder()
            .headless_mode(HeadlessMode::True)
            .args(["--no-sandbox", "--disable-dev-shm-usage"])
            .build()
            .map_err(|e| ScraperInitError::Browser(e.to_string()))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| ScraperInitError::Browser(e.to_string()))?;

        debug!("Browser launched");

        // Making sure the browser is ready for stuff
        sleep(std::time::Duration::from_secs(5)).await;

        let handler_task = tokio::spawn(async move { while let Some(_) = handler.next().await {} });

        let my_browser = Scraper {
            browser,
            handler_task,
        };

        Ok(my_browser)
    }

    // TODO better error handling
    pub async fn get_players_stats(
        &mut self,
        registered_players: &Vec<RegisteredPlayer>,
    ) -> Vec<PlayerWithStats> {
        let mut players_stats: Vec<PlayerWithStats> = vec![];
        for player in registered_players {
            if let Ok(player_stat) = self.get_player_stats(&player.rematch_url).await {
                players_stats.push(PlayerWithStats {
                    discord_id: player.discord_id,
                    display_name: player_stat.player.display_name,
                    rank: player_stat.rank,
                });
            } else {
                error!("Failed to fetch stats for {}", player.discord_id);
            }
        }

        players_stats
    }

    async fn get_player_stats(&mut self, url: &str) -> Result<PlayerStat, ScrapeError> {
        debug!("get_player_stats - start for url={}", url);
        let page = self
            .browser
            .new_page("about:blank")
            .await
            .map_err(|e| ScrapeError::PageInit(e.to_string()))?;

        page.execute(NetworkEnableParams::default())
            .await
            .map_err(|e| ScrapeError::PageInit(e.to_string()))?;

        let mut responses = page
            .event_listener::<EventRequestWillBeSent>()
            .await
            .map_err(|e| ScrapeError::PageInit(e.to_string()))?;
        let mut finished_events = page
            .event_listener::<EventLoadingFinished>()
            .await
            .map_err(|e| ScrapeError::PageInit(e.to_string()))?;

        page.goto(url)
            .await
            .map_err(|e| ScrapeError::PageInit(e.to_string()))?;
        debug!("get_player_stats - browser on page url={}", url);

        let result = async {
            let mut api_profile_request_id = None;
            while let Ok(Some(event)) = timeout(Duration::from_secs(10), responses.next()).await {
                let method = &event.request.method;
                let url = &event.request.url;

                if method == "GET" && url.contains("/api/profiles") {
                    api_profile_request_id = Some(event.request_id.clone());
                    debug!("Profile event url={:?} id={:?}", url, event.request_id);
                    break;
                }
            }

            let api_profile_request_id = match api_profile_request_id {
                Some(id) => id,
                None => {
                    error!("The user profile request was not found.");
                    return Err(ScrapeError::RequestNotFound);
                }
            };

            while let Ok(Some(event)) =
                timeout(Duration::from_secs(10), finished_events.next()).await
            {
                if api_profile_request_id == event.request_id {
                    debug!("Found matching event: {:?}", event);
                    let response_body = page
                        .execute(GetResponseBodyParams::new(event.request_id.clone()))
                        .await
                        .map_err(|_| ScrapeError::RequestContent)?;

                    let body = response_body.body.clone();
                    debug!("event {:?} content: {}", event.request_id, body);

                    let player_stat: PlayerStat = serde_json::from_str(&body).unwrap();
                    return Ok(player_stat);
                }
            }

            return Err(ScrapeError::Timeout);
        }
        .await;

        let _ = page.close().await;

        result
    }
}
