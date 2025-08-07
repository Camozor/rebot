use std::sync::Arc;

use log::{debug, info};
use rebot::{
    config::Config, discord::Discord, player_store::PlayerStore, server::start_http_server,
};
use tokio::sync::Mutex;

use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    let store = PlayerStore::load_database(&config);
    let store = Arc::new(Mutex::new(store));
    let cron_store = store.clone();
    let mut discord = Discord::new(store).await;

    tokio::select! {
        _ = start_http_server(&config, discord.get_context())
            .launch() => {
            info!("Server stopped.");
        }
        _ = discord.start() => {
            info!("Discord bot stopped.");
        }
        _ = cron_refresh(cron_store) => {
            info!("Refresh cron stopped.");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received. Shutting down...");
        }
    }

    Ok(())
}

async fn cron_refresh(store: Arc<Mutex<PlayerStore>>) {
    let interval_second = {
        let store = store.lock().await;
        store.config.cron_interval_minute as u64 * 60
    };
    let mut interval = time::interval(Duration::new(interval_second, 0));
    interval.tick().await;

    loop {
        let execute_cron = !store.lock().await.config.skip_cron;
        if execute_cron {
            debug!("Start refresh");
            let _ = store.lock().await.refresh_all().await;
            debug!("End refresh");
        }

        interval.tick().await;
        debug!("End wait");
    }
}
