use chromiumoxide::cdp::browser_protocol::network::{
    EnableParams as NetworkEnableParams, EventRequestWillBeSent,
};
use chromiumoxide::{
    Browser, BrowserConfig, browser::HeadlessMode,
    cdp::browser_protocol::network::EventLoadingFinished,
};
use tokio::task::spawn_blocking;
use tokio::time::{Duration, sleep, timeout};

use chromiumoxide::cdp::browser_protocol::network::GetResponseBodyParams;
use futures::StreamExt;
use log::{debug, error, info};

use crate::model::player_stat::PlayerStat;

pub async fn compute_all_stats() {
    let mut browser = MyBrowser::new().await.expect("Could not create browser");
    let stats = browser.get_player_stats("https://u.gg/rematch/profile/steam/La%20m%C3%A9sange%20du%20Val%20d'Oise/76561198355389674").await.expect("Error stats");
    info!("Computed stats: {:?}", stats);
    browser.close().await;
}

struct MyBrowser {
    browser: Browser,
}

impl MyBrowser {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Browser creation");
        let config = BrowserConfig::builder()
            .headless_mode(HeadlessMode::True)
            .build()?;

        let (browser, mut handler) = Browser::launch(config).await?;

        // Making sure the browser is ready for stuff
        sleep(std::time::Duration::from_secs(5)).await;

        spawn_blocking(move || {
            futures::executor::block_on(async {
                while let Some(event) = handler.next().await {
                    if let Err(e) = event {
                        error!("Future handler error: {:?}", e);
                    }
                }
            });
        });

        let my_browser = MyBrowser { browser };

        Ok(my_browser)
    }

    async fn get_player_stats(
        &mut self,
        url: &str,
    ) -> Result<PlayerStat, Box<dyn std::error::Error>> {
        let page = self.browser.new_page("about:blank").await?;

        page.execute(NetworkEnableParams::default()).await?;

        let mut responses = page.event_listener::<EventRequestWillBeSent>().await?;
        let mut finished_events = page.event_listener::<EventLoadingFinished>().await?;

        page.goto(url).await?;

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
                    return Err("The user profile request was not found.".into());
                }
            };

            while let Ok(Some(event)) =
                timeout(Duration::from_secs(10), finished_events.next()).await
            {
                if api_profile_request_id == event.request_id {
                    debug!("Found matching event: {:?}", event);
                    let response_body = page
                        .execute(GetResponseBodyParams::new(event.request_id.clone()))
                        .await?;

                    let body = response_body.body.clone();
                    debug!("event {:?} content: {}", event.request_id, body);

                    let player_stat: PlayerStat = serde_json::from_str(&body).unwrap();
                    return Ok(player_stat);
                }
            }

            return Err("Timed out waiting for requests".into());
        }
        .await;

        page.close().await?;

        result
    }

    async fn close(&mut self) {
        let _ = self.browser.close().await;
    }
}
