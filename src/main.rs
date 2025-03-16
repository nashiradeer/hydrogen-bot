use handler::{handle_command, handle_component, register_commands};
use lavalink::{Rest, cluster::Cluster};
use music::PlayerManager;
use parking_lot::Mutex;
use serenity::{
    all::{
        Client, CommandId, GatewayIntents, Interaction, Ready, VoiceServerUpdateEvent, VoiceState,
    },
    client::{Context, EventHandler},
};
use songbird::SerenityInit;
use std::{
    collections::HashMap,
    env,
    process::exit,
    sync::{Arc, OnceLock},
    time::Instant,
};
use tracing::{Level, event, instrument};
use tracing_subscriber::{
    EnvFilter, fmt::layer, layer::SubscriberExt, registry, util::SubscriberInitExt,
};
use utils::constants::{
    HYDROGEN_UPDATE_VOICE_SERVER_THRESHOLD, HYDROGEN_UPDATE_VOICE_STATE_THRESHOLD,
};

mod commands;
mod components;
mod handler;
mod i18n;
#[allow(dead_code)]
mod lavalink;
mod music;
mod shared;
mod utils;

/// The commands IDs that are registered.
pub static LOADED_COMMANDS: OnceLock<HashMap<String, CommandId>> = OnceLock::new();

/// Hydrogen's Player Manager.
pub static PLAYER_MANAGER: OnceLock<PlayerManager> = OnceLock::new();

/// The program's entry point.
fn main() {
    registry()
        .with(layer())
        .with(EnvFilter::from_default_env())
        .init();

    let disable_multi_threading = env::var("DISABLE_MULTI_THREADING").is_ok_and(|v| v == "true");

    let mut tokio_runtime_builder = if disable_multi_threading {
        event!(Level::INFO, "multi-threading is disabled");
        tokio::runtime::Builder::new_current_thread()
    } else {
        event!(Level::INFO, "multi-threading is enabled");
        tokio::runtime::Builder::new_multi_thread()
    };

    let tokio_runtime = match tokio_runtime_builder.enable_all().build() {
        Ok(v) => v,
        Err(e) => {
            event!(Level::ERROR, error = ?e, "cannot create the Tokio runtime");
            exit(1);
        }
    };

    tokio_runtime.block_on(hydrogen());
}

/// Hydrogen's entry point.
async fn hydrogen() {
    let lavalink_nodes = init_lavalink();

    if lavalink_nodes.is_empty() {
        event!(Level::ERROR, "no Lavalink nodes were found");
        exit(1);
    }

    let discord_token = match env::var("DISCORD_TOKEN") {
        Ok(v) => v,
        Err(e) => {
            event!(Level::ERROR, error = ?e, "cannot get the Discord token");
            exit(1);
        }
    };

    let mut client = match Client::builder(
        &discord_token,
        GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS,
    )
    .event_handler(HydrogenHandler {
        lavalink_nodes: Mutex::new(Some(lavalink_nodes)),
    })
    .register_songbird()
    .await
    {
        Ok(v) => v,
        Err(e) => {
            event!(Level::ERROR, error = ?e, "cannot create the client");
            exit(1);
        }
    };

    match client.start().await {
        Ok(_) => (),
        Err(e) => {
            event!(Level::ERROR, error = ?e, "cannot start the client");
            exit(1);
        }
    }
}

/// Initializes the Lavalink nodes.
fn init_lavalink() -> Vec<Rest> {
    let lavalink_builder = match lavalink::hydrogen::ConfigParser::new() {
        Ok(v) => v,
        Err(e) => {
            event!(Level::ERROR, error = ?e, "cannot create the Lavalink builder");
            exit(1);
        }
    };

    let lavalink = match env::var("LAVALINK") {
        Ok(v) => v,
        Err(e) => {
            event!(Level::ERROR, error = ?e, "cannot get the Lavalink URL");
            exit(1);
        }
    };

    lavalink_builder.parse(&lavalink)
}

/// The Hydrogen handler.
pub struct HydrogenHandler {
    /// The Lavalink nodes.
    lavalink_nodes: Mutex<Option<Vec<Rest>>>,
}

#[serenity::async_trait]
impl EventHandler for HydrogenHandler {
    #[instrument(skip_all)]
    /// Handles the ready event.
    async fn ready(&self, ctx: Context, ready: Ready) {
        event!(Level::DEBUG, "initializing Hydrogen...");
        let init_time = Instant::now();

        let Some(songbird) = songbird::get(&ctx).await else {
            event!(Level::ERROR, "songbird is not initialized");
            exit(1);
        };

        let Some(lavalink_nodes) = self.lavalink_nodes.lock().take() else {
            event!(Level::ERROR, "Lavalink nodes are not initialized");
            exit(1);
        };

        event!(
            Level::INFO,
            node_count = lavalink_nodes.len(),
            "connecting to Lavalink nodes..."
        );

        if PLAYER_MANAGER
            .set(
                PlayerManager::new(
                    songbird,
                    Arc::new(Cluster::new(lavalink_nodes, &ready.user.id.to_string()).await),
                    ctx.cache.clone(),
                    ctx.http.clone(),
                )
                .await,
            )
            .is_err()
        {
            event!(Level::ERROR, "cannot set the PlayerManager");
            exit(1);
        }

        if !register_commands(&ctx.http).await {
            exit(1);
        }

        let exec_time = init_time.elapsed();
        if exec_time > utils::constants::HYDROGEN_READY_THRESHOLD {
            event!(Level::WARN, time = ?exec_time, user_name = %ready.user.name, "initializing Hydrogen took too long");
        } else {
            event!(Level::INFO, time = ?exec_time, user_name = %ready.user.name, "initialized Hydrogen");
        }
    }

    #[instrument(skip_all, fields(guild_id = ?voice_server.guild_id.map(|v| v.get())))]
    /// Handles the voice server update event.
    async fn voice_server_update(&self, _ctx: Context, voice_server: VoiceServerUpdateEvent) {
        event!(Level::DEBUG, "updating voice server...");
        let init_time = Instant::now();

        if let Some(manager) = PLAYER_MANAGER.get() {
            if let Err(e) = manager.update_voice_server(voice_server).await {
                event!(Level::ERROR, error = ?e, "cannot update the voice server");
            }
        }

        let exec_time = init_time.elapsed();
        if exec_time > HYDROGEN_UPDATE_VOICE_SERVER_THRESHOLD {
            event!(
                Level::WARN,
                time = ?exec_time,
                "updating the voice server took too long"
            );
        } else {
            event!(
                Level::INFO,
                time = ?exec_time,
                "voice server updated"
            );
        }
    }

    #[instrument(skip_all, fields(user_id = %new.user_id, guild_id = ?new.guild_id.map(|v| v.get()), channel_id = ?new.channel_id.map(|v| v.get())))]
    /// Handles the voice state update event.
    async fn voice_state_update(&self, _ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        event!(Level::DEBUG, "updating voice state...");
        let init_time = Instant::now();

        if let Some(manager) = PLAYER_MANAGER.get() {
            if let Err(e) = manager.update_voice_state(old.as_ref(), &new).await {
                event!(Level::ERROR, error = ?e, "cannot update the voice state");
            }
        }

        let exec_time = init_time.elapsed();
        if exec_time > HYDROGEN_UPDATE_VOICE_STATE_THRESHOLD {
            event!(
                Level::WARN,
                time = ?exec_time,
                "updating the voice state took too long"
            );
        } else {
            event!(
                Level::INFO,
                time = ?exec_time,
                "voice state updated"
            );
        }
    }

    #[instrument(skip_all, fields(interaction.id = %interaction.id(), interaction.kind = ?interaction.kind()))]
    /// Handles the interaction create event.
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        event!(Level::DEBUG, "handling interaction...");
        let init_time = Instant::now();

        match interaction {
            Interaction::Command(command) => handle_command(&ctx, &command).await,
            Interaction::Component(component) => handle_component(&ctx, &component).await,
            _ => (),
        }

        let exec_time = init_time.elapsed();
        if exec_time > utils::constants::HYDROGEN_INTERACTION_CREATE_THRESHOLD {
            event!(
                Level::WARN,
                time = ?exec_time,
                "handling the interaction took too long"
            );
        } else {
            event!(
                Level::INFO,
                time = ?exec_time,
                "interaction handled"
            );
        }
    }
}
