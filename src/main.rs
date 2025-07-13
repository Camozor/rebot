use log::{debug, info};
use rebot::{discord::Discord, player_store::PlayerStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(_) = dotenvy::dotenv() {
        debug!(".env file not found");
    }
    env_logger::init();

    let store = PlayerStore::with_data();
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
