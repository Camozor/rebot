use futures::lock::Mutex;
use log::{debug, info};
use poise::serenity_prelude::{self as serenity, GuildId};
use std::env;

use crate::player_store::{PlayerStore, RegisterError};

struct DiscordState {
    pub player_store: Mutex<PlayerStore>,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, DiscordState, Error>;

#[poise::command(slash_command)]
async fn register(
    ctx: Context<'_>,
    #[description = "URL rematch"] rematch_url: String,
) -> Result<(), Error> {
    let u = ctx.author();

    info!(
        "Register command for user id={} with url={}",
        u.id, rematch_url
    );

    let mut player_store = ctx.data().player_store.lock().await;
    let register_status = player_store.register_player(u.id.into(), &rematch_url);

    let response = match register_status {
        Ok(_) => format!("Très bien {}, on a enregistré ta page u.gg!", u.name),
        Err(RegisterError::WrongUrl(e)) => format!("Fais un effort {}, {}", u.name, e),
    };
    debug!(
        "Now there is {} players in the store {:?}",
        player_store.registered_players.len(),
        player_store.registered_players
    );
    ctx.say(response).await?;
    Ok(())
}

pub struct Discord {
    client: serenity::Client,
}

impl Discord {
    pub async fn new(store: PlayerStore) -> Self {
        info!("Configuring discord bot");
        let discord_token = env::var("DISCORD_TOKEN").expect("Configure your discord token bro!");
        let intents = serenity::GatewayIntents::non_privileged();

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![register()],
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                let discord_server_id = env::var("DISCORD_SERVER_ID");

                let mutex_store = Mutex::new(store);
                if discord_server_id.is_ok() {
                    debug!("Discord commands registered in dev mode");
                    Box::pin(async move {
                        let server_id = discord_server_id.unwrap();
                        let server_id = server_id
                            .parse::<u64>()
                            .expect("A discord server id is a number bro!");
                        poise::builtins::register_in_guild(
                            ctx,
                            &framework.options().commands,
                            GuildId::new(server_id),
                        )
                        .await?;
                        Ok(DiscordState {
                            player_store: mutex_store,
                        })
                    })
                } else {
                    debug!("Discord commands registered in production mode");
                    Box::pin(async move {
                        poise::builtins::register_globally(ctx, &framework.options().commands)
                            .await?;
                        Ok(DiscordState {
                            player_store: mutex_store,
                        })
                    })
                }
            })
            .build();

        let client = serenity::ClientBuilder::new(discord_token, intents)
            .framework(framework)
            .await
            .expect("Could not instantiate discord client");
        Discord { client: client }
    }

    pub async fn start(&mut self) {
        info!("Starting discord bot");
        self.client.start().await.unwrap();
    }
}
