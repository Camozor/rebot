use log::info;
use rebot::{config::Config, discord::Discord, player_store::PlayerStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    let store = PlayerStore::load_database(config);
    let mut discord = Discord::new(store).await;

    tokio::select! {
        _ = discord.start() => {
            info!("Discord bot stopped.");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received. Shutting down...");
        }
    }

    Ok(())
}
