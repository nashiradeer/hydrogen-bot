use std::{
    collections::HashMap,
    env,
    process::exit,
    sync::{LazyLock, OnceLock},
    time::Instant,
};

use async_trait::async_trait;
use dashmap::DashMap;
use handler::{handle_command, handle_component, register_commands, AutoRemoverKey};
use lavalink::LavalinkNodeInfo;
use manager::HydrogenManager;
use serenity::{
    all::{
        Client, CommandId, ComponentInteraction, GatewayIntents, Interaction, Ready,
        VoiceServerUpdateEvent, VoiceState,
    },
    client::{Context, EventHandler},
};
use songbird::SerenityInit;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{
    fmt::layer, layer::SubscriberExt, registry, util::SubscriberInitExt, EnvFilter,
};

mod commands;
mod components;
mod handler;
mod i18n;
pub mod lavalink;
mod manager;
mod player;
mod utils;

/// The commands IDs that are registered.
pub static LOADED_COMMANDS: OnceLock<HashMap<String, CommandId>> = OnceLock::new();

/// Hydrogen's Player Manager.
pub static MANAGER: OnceLock<HydrogenManager> = OnceLock::new();

/// The messages from the components.
pub static COMPONENTS_MESSAGES: LazyLock<
    DashMap<AutoRemoverKey, (JoinHandle<()>, ComponentInteraction)>,
> = LazyLock::new(DashMap::new);

/// Lavalink node.
pub static LAVALINK_NODE: OnceLock<LavalinkNodeInfo> = OnceLock::new();

#[tokio::main]
/// Hydrogen's main function.
async fn main() {
    registry()
        .with(layer())
        .with(EnvFilter::from_default_env())
        .init();

    LAVALINK_NODE
        .set(LavalinkNodeInfo {
            host: env::var("LAVALINK_HOST").expect("LAVALINK_HOST is not set or invalid unicode"),
            password: env::var("LAVALINK_PASSWORD")
                .expect("LAVALINK_PASSWORD is not set or invalid unicode"),
            tls: env::var_os("LAVALINK_TLS").is_some(),
        })
        .expect("cannot set LAVALINK_NODE");

    let mut client = Client::builder(
        env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set or invalid unicode"),
        GatewayIntents::GUILDS
            | GatewayIntents::GUILD_VOICE_STATES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler(HydrogenHandler)
    .register_songbird()
    .await
    .expect("cannot initialize client");

    client.start().await.expect("cannot start client");
}

pub struct HydrogenHandler;

#[async_trait]
impl EventHandler for HydrogenHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let timer = Instant::now();
        debug!("(ready): processing...");

        if MANAGER
            .set(HydrogenManager::new(ctx.cache.clone(), ctx.http.clone()))
            .is_err()
        {
            error!("(ready): cannot set HydrogenManager in OnceLock");
            exit(1);
        }

        debug!("(ready): HydrogenManager initialized");

        if !register_commands(&ctx.http).await {
            error!("(ready): cannot register commands");
            exit(1);
        }

        let Some(manager) = MANAGER.get() else {
            error!("(ready): cannot get HydrogenManager from OnceLock");
            exit(1);
        };

        let Some(lavalink_node) = LAVALINK_NODE.get() else {
            error!("(ready): cannot get LavalinkNodeInfo from OnceLock");
            exit(1);
        };

        if manager
            .connect_lavalink(lavalink_node.clone())
            .await
            .is_err()
        {
            error!("(ready): cannot connect to Lavalink");
            exit(1);
        }

        info!(
            "(ready): client connected to '{}' in {}ms",
            ready.user.name,
            timer.elapsed().as_millis()
        );
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let timer = Instant::now();
        debug!("(interaction_create): processing...");

        match interaction {
            Interaction::Command(command) => {
                handle_command(&ctx, &command).await;

                info!(
                    "(interaction_create): command '{}' executed in {}ms",
                    command.data.name,
                    timer.elapsed().as_millis()
                );
            }
            Interaction::Component(component) => {
                handle_component(&ctx, &component).await;

                info!(
                    "(interaction_create): component '{}' executed in {}ms",
                    component.data.custom_id,
                    timer.elapsed().as_millis()
                );
            }
            _ => (),
        }
    }

    async fn voice_state_update(&self, _: Context, old: Option<VoiceState>, new: VoiceState) {
        let timer = Instant::now();
        debug!("(voice_state_update): processing...");

        if let Some(manager) = MANAGER.get() {
            match manager.update_voice_state(old, new).await {
                Ok(updated) => {
                    if updated {
                        info!(
                            "(voice_state_update): processed in {}ms...",
                            timer.elapsed().as_millis()
                        );
                    } else {
                        debug!("(voice_state_update): ignored");
                    }
                }
                Err(e) => {
                    warn!("(voice_state_update): cannot update the HydrogenManager's player voice state: {}", e);
                }
            }
        }
    }

    async fn voice_server_update(&self, _: Context, voice_server: VoiceServerUpdateEvent) {
        let timer = Instant::now();
        debug!("(voice_server_update): processing...");

        if let Some(manager) = MANAGER.get() {
            match manager.update_voice_server(voice_server).await {
                Ok(updated) => {
                    if updated {
                        info!(
                            "(voice_server_update): processed in {}ms...",
                            timer.elapsed().as_millis()
                        );
                    } else {
                        debug!("(voice_server_update): ignored");
                    }
                }
                Err(e) => {
                    warn!("(voice_server_update): cannot update HydrogenManager's player voice server: {}", e);
                }
            }
        }
    }
}
