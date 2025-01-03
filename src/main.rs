use std::{
    collections::HashMap,
    env,
    process::exit,
    sync::{Arc, LazyLock, OnceLock},
};

use async_trait::async_trait;
use dashmap::DashMap;
use handler::{handle_command, register_commands, AutoRemoverKey};
use lavalink::{cluster::Cluster, Rest};
use music::PlayerManager;
use parking_lot::Mutex;
use serenity::{
    all::{
        Client, CommandId, ComponentInteraction, GatewayIntents, Interaction, Ready,
        VoiceServerUpdateEvent, VoiceState,
    },
    client::{Context, EventHandler},
};
use songbird::SerenityInit;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, info_span, warn, Span};
use tracing_subscriber::{
    fmt::layer, layer::SubscriberExt, registry, util::SubscriberInitExt, EnvFilter,
};

mod commands;
//mod components;
mod handler;
mod i18n;
pub mod lavalink;
mod music;
mod utils;

/// The commands IDs that are registered.
pub static LOADED_COMMANDS: OnceLock<HashMap<String, CommandId>> = OnceLock::new();

/// Hydrogen's Player Manager.
pub static PLAYER_MANAGER: OnceLock<PlayerManager> = OnceLock::new();

/// The messages from the components.
pub static COMPONENTS_MESSAGES: LazyLock<
    DashMap<AutoRemoverKey, (JoinHandle<()>, ComponentInteraction)>,
> = LazyLock::new(DashMap::new);

#[tokio::main]
/// Hydrogen's main function.
async fn main() {
    registry()
        .with(layer())
        .with(EnvFilter::from_default_env())
        .init();

    let lavalink_nodes = init_lavalink();

    if lavalink_nodes.is_empty() {
        panic!("no Lavalink nodes found");
    }

    let mut client = Client::builder(
        env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set or invalid unicode"),
        GatewayIntents::GUILDS
            | GatewayIntents::GUILD_VOICE_STATES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler(HydrogenHandler {
        lavalink_nodes: Mutex::new(Some(lavalink_nodes)),
        ready_span: info_span!("ready"),
        interaction_span: info_span!("interaction"),
        voice_state_span: info_span!("voice_state"),
        voice_server_span: info_span!("voice_server"),
    })
    .register_songbird()
    .await
    .expect("cannot initialize client");

    client.start().await.expect("cannot start client");
}

fn init_lavalink() -> Vec<Rest> {
    let lavalink_builder =
        lavalink::hydrogen::ConfigParser::new().expect("cannot create ConfigParser");

    lavalink_builder.parse(env::var("LAVALINK").expect("LAVALINK is not set or invalid unicode"))
}

pub struct HydrogenHandler {
    lavalink_nodes: Mutex<Option<Vec<Rest>>>,
    ready_span: Span,
    interaction_span: Span,
    voice_state_span: Span,
    voice_server_span: Span,
}

#[async_trait]
impl EventHandler for HydrogenHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let _entered = self.ready_span.enter();

        let Some(songbird) = songbird::get(&ctx).await else {
            error!("(ready): cannot get Songbird from Context, is it initialized?");
            exit(1);
        };

        let Some(lavalink_nodes) = self.lavalink_nodes.lock().take() else {
            error!("(ready): cannot get the Lavalink nodes, is it initialized?");
            exit(1);
        };

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
            error!("(ready): cannot set PlayerManager in OnceLock");
            exit(1);
        }

        debug!("(ready): PlayerManager initialized");

        if !register_commands(&ctx.http).await {
            error!("(ready): cannot register commands");
            exit(1);
        }

        info!("(ready): client connected to '{}'", ready.user.name,);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let _entered = self.interaction_span.enter();

        match interaction {
            Interaction::Command(command) => {
                info!(
                    "(interaction_create): executing command '{}'...",
                    command.data.name,
                );

                handle_command(&ctx, &command).await;
            }
            Interaction::Component(component) => {
                info!(
                    "(interaction_create): executing component '{}'...",
                    component.data.custom_id,
                );

                //handle_component(&ctx, &component).await;
            }
            _ => (),
        }
    }

    async fn voice_state_update(&self, _: Context, old: Option<VoiceState>, new: VoiceState) {
        let _entered = self.voice_state_span.enter();

        if let Some(manager) = PLAYER_MANAGER.get() {
            if let Err(e) = manager.update_voice_state(old, new).await {
                warn!("(voice_state_update): cannot update the HydrogenManager's player voice state: {}", e);
            }
        }
    }

    async fn voice_server_update(&self, _: Context, voice_server: VoiceServerUpdateEvent) {
        let _entered = self.voice_server_span.enter();

        if let Some(manager) = PLAYER_MANAGER.get() {
            if let Err(e) = manager.update_voice_server(voice_server).await {
                warn!("(voice_server_update): cannot update HydrogenManager's player voice server: {}", e);
            }
        }
    }
}
