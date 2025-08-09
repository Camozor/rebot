use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use log::{debug, error, info};
use poise::serenity_prelude::{self as serenity, GuildId};
use serenity::all::GatewayIntents;
use tokio::{sync::Mutex, sync::RwLock, time::timeout};

use crate::player_store::{PlayerStore, RegisterError};

struct DiscordState {
    pub player_store: Arc<Mutex<PlayerStore>>,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, DiscordState, Error>;

use songbird::{input::File, Event, EventContext, EventHandler, SerenityInit, TrackEvent};

struct TrackEndNotifier {
    notify: Arc<tokio::sync::Notify>,
}

#[async_trait::async_trait]
impl EventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            self.notify.notify_one();
        }
        None
    }
}

#[poise::command(slash_command)]
async fn marius(ctx: Context<'_>) -> Result<(), Error> {
    let context = ctx.serenity_context();
    let user_id = ctx.author().id;
    let guild_id = ctx.guild_id().unwrap();

    play_marius(context, guild_id, user_id).await
}

pub async fn play_marius(
    serenity_context: &serenity::prelude::Context,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("guild_id={}", guild_id);
    info!("user_id={}", user_id);
    let channel_id = {
        let cache = &serenity_context.cache;
        let guild = cache.guild(guild_id).ok_or("Guild not found in cache")?;
        let voice_state = guild
            .voice_states
            .get(&user_id)
            .ok_or("Rejoins le serveur vocal avant d'exécuter cette commande ! 😉")?;
        voice_state
            .channel_id
            .ok_or("No channel ID found in voice state")?
    };

    let manager = songbird::get(serenity_context)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let join = manager.join(guild_id, channel_id).await?;

    let source = File::new("audio/mario.mp3");

    let mut handle = join.lock().await;

    let track_end_notify = Arc::new(tokio::sync::Notify::new());
    let handler_notify = track_end_notify.clone();
    handle.add_global_event(
        Event::Track(TrackEvent::End),
        TrackEndNotifier {
            notify: handler_notify,
        },
    );
    handle.play_input(source.into());

    let _ = timeout(Duration::from_secs(30), track_end_notify.notified()).await;

    let _ = handle.leave().await;

    Ok(())
}

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

    let now = Instant::now();
    if let Err(_) = player_store.refresh_all().await {
        error!("Could not refresh");
    }

    player_store.print();

    info!("Players refresh success in {}s", now.elapsed().as_secs());

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
            "**{}** aussi connu sous le nom **{}** est rang **{}**, **{}**",
            compute_pretty_player_name(&u.name),
            player.display_name,
            player.rank.pretty_rank(),
            player.get_pretty_stats()
        ),
    };

    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    let u = ctx.author();
    info!(
        "Stats command for author id={}, target user id={}",
        ctx.author().id,
        u.id
    );

    let player_store = ctx.data().player_store.lock().await;
    let all_players = player_store.get_all_players_stat();

    let stats: Vec<String> = all_players
        .iter()
        .map(|player| {
            format!(
                "* {} - {} - {}",
                compute_pretty_player_name(&player.display_name),
                player.rank.pretty_rank(),
                player.get_pretty_stats()
            )
        })
        .collect();
    let players_pretty_stat = stats.join("\n");

    let try_hard_player = player_store.find_try_harder();

    let response = match try_hard_player {
        None => players_pretty_stat,
        Some(p) => {
            let player = compute_pretty_player_name(&p.display_name);
            let hours_played = p.estimate_hours_played();
            format!(
                "{}\nEt la palme d'or du plus gros try harder revient à {} avec plus de {}h de jeu !",
                players_pretty_stat, player, hours_played,
            )
        }
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
    data: SharedData,
}

#[derive(Clone)]
struct SharedData {
    discord_ctx: Arc<RwLock<Option<Arc<serenity::prelude::Context>>>>,
}

struct Handler {
    shared: SharedData,
}

#[serenity::async_trait]
impl serenity::client::EventHandler for Handler {
    async fn ready(
        &self,
        ctx: poise::serenity_prelude::Context,
        _: serenity::model::gateway::Ready,
    ) {
        info!("Bot is connected!");
        let mut context_lock = self.shared.discord_ctx.write().await;
        *context_lock = Some(Arc::new(ctx));
    }
}

impl Discord {
    pub async fn new(store: Arc<Mutex<PlayerStore>>) -> Self {
        info!("Configuring discord bot");
        let config = store.lock().await.config.clone();
        let intents = GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS;

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![register(), refresh(), stat(), stats(), marius()],
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                let discord_server_id = config.discord_server_id;

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
                            player_store: store,
                        })
                    })
                } else {
                    debug!("Discord commands registered in production mode");
                    Box::pin(async move {
                        poise::builtins::register_globally(ctx, &framework.options().commands)
                            .await?;
                        Ok(DiscordState {
                            player_store: store,
                        })
                    })
                }
            })
            .build();

        let shared = SharedData {
            discord_ctx: Arc::new(RwLock::new(None)),
        };

        let handler = Handler {
            shared: shared.clone(),
        };

        let client = serenity::ClientBuilder::new(config.discord_token, intents)
            .framework(framework)
            .event_handler(handler)
            .register_songbird()
            .await
            .expect("Could not instantiate discord client");

        Discord {
            client,
            data: shared,
        }
    }

    pub async fn start(&mut self) {
        info!("Starting discord bot");
        self.client.start().await.unwrap();
    }

    pub fn get_context(&self) -> Arc<RwLock<Option<Arc<serenity::prelude::Context>>>> {
        self.data.discord_ctx.clone()
    }
}
