use rebot::{discord::Discord, player_store::PlayerStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    env_logger::init();

    // compute_all_stats().await;

    let store = PlayerStore::new();

    let mut discord = Discord::new(store).await;
    discord.start().await;

    Ok(())
}
