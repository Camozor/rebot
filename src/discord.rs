use futures::lock::Mutex;
use log::{debug, info};
use poise::serenity_prelude::{self as serenity, GuildId};

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
        Ok(_) => format!("Très bien {}, on a enregistré ta page u.gg !", u.name),
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

#[poise::command(slash_command)]
async fn refresh(ctx: Context<'_>) -> Result<(), Error> {
    let u = ctx.author();
    info!("Refresh command for user id={}", u.id);

    let response = format!(
        "On démarre le scraping intensif, ça peut prendre quelques secondes. SVP u.gg ne portez pas plainte !"
    );
    ctx.say(response).await?;

    let mut player_store = ctx.data().player_store.lock().await;
    let _ = player_store.refresh_all().await;

    debug!("Player store state: {:?}", player_store);

    Ok(())
}

#[poise::command(slash_command)]
async fn stat(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    info!(
        "Stat command for author id={}, target user id={}",
        ctx.author().id,
        u.id
    );

    let player_store = ctx.data().player_store.lock().await;
    let player_stat = player_store.get_player_stat(u.id.into());

    let response = match player_stat {
        None => format!(
            "{} n'est pas enregistré, pense à utiliser la commande /register",
            u.name
        ),
        Some(player) => format!(
            "**{}** aussi connu sous le nom **{}** est rang **{}**",
            compute_pretty_player_name(&u.name),
            player.display_name,
            player.rank.pretty_rank()
        ),
    };

    ctx.say(response).await?;
    Ok(())
}

fn compute_pretty_player_name(name: &str) -> String {
    let mut c = name.chars();
    match c.next() {
        None => String::new(),
        Some(first_char) => first_char.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub struct Discord {
    client: serenity::Client,
}

impl Discord {
    pub async fn new(store: PlayerStore) -> Self {
        info!("Configuring discord bot");
        let config = store.config.clone();
        let intents = serenity::GatewayIntents::non_privileged();

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![register(), refresh(), stat()],
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                let discord_server_id = config.discord_server_id;

                let mutex_store = Mutex::new(store);
                if discord_server_id.is_some() {
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

        let client = serenity::ClientBuilder::new(config.discord_token, intents)
            .framework(framework)
            .await
            .expect("Could not instantiate discord client");
        Discord { client }
    }

    pub async fn start(&mut self) {
        info!("Starting discord bot");
        self.client.start().await.unwrap();
    }
}
