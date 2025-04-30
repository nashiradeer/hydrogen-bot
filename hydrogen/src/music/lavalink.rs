use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use hydrolink::{Event, Message, MessageKind, cluster::Cluster};
use serenity::all::GuildId;
use tokio::time::sleep;
use tracing::{Instrument, Level, event, instrument, span};

use super::PlayerManager;
use crate::utils::constants::{HYDROGEN_LAVALINK_EVENT_THRESHOLD, LAVALINK_RECONNECTION_DELAY};

/// Handle the Lavalink events.
pub fn handle_lavalink(player_manager: PlayerManager) {
    tokio::spawn(async move {
        while let Some((node_id, message)) = player_manager.clone().lavalink.recv().await {
            let player_manager = player_manager.clone();

            tokio::spawn(async move {
                process_message(player_manager, node_id, message).await;
            });
        }
    });
}

#[instrument(skip_all, parent = None, name = "lavalink_handler", fields(node_id = node_id, message_kind = ?message.as_ref().and_then(|v| v.as_ref().ok().map(|v2| v2.kind())), event_kind = ?message.as_ref().and_then(|v| v.as_ref().ok().and_then(|v2| v2.as_event().map(|v3| v3.kind()))), guild_id = ?message.as_ref().and_then(|v| v.as_ref().map(|v2| v2.guild_id()).ok()).flatten()))]
/// Process the message from Lavalink.
async fn process_message(
    player_manager: PlayerManager,
    node_id: usize,
    message: Option<Result<Message, hydrolink::Error>>,
) {
    let spammy_message = message
        .as_ref()
        .map(|s| {
            s.as_ref()
                .map(|v| {
                    let kind = v.kind();

                    kind == MessageKind::Stats || kind == MessageKind::PlayerUpdate
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    event!(Level::DEBUG, "handling Lavalink message");
    let init_time = Instant::now();

    if let Some(ref message) = message {
        match message {
            Ok(data) => {
                event!(Level::TRACE, message = ?data);

                let player_manager = player_manager.clone();

                process_data(data, &player_manager).await;
            }
            Err(e) => event!(
                Level::ERROR,
                error = ?e,
                "failed to receive Lavalink message"
            ),
        }
    } else {
        event!(Level::WARN, "Lavalink has disconnected");

        let mut should_remove = false;

        let mut restart_player_list = Vec::new();

        for mut player in player_manager.players.iter_mut() {
            let player_key = *player.key();
            let player_value = player.value_mut();

            if player_value.node_id == node_id {
                if let Some(node_id) = player_manager.lavalink.search_connected_node() {
                    event!(
                        Level::DEBUG,
                        old_node = player_value.node_id,
                        new_node = node_id,
                        "migrating player..."
                    );
                    player_value.node_id = node_id;

                    restart_player_list.push(player_key);
                } else {
                    event!(
                        Level::ERROR,
                        "there's no available Lavalink to migrate, removing players"
                    );
                    should_remove = true;
                    break;
                }
            }
        }

        for player_key in restart_player_list {
            if let Err(e) = player_manager.sync(player_key).await {
                event!(
                    Level::ERROR,
                    guild_id = %player_key,
                    error = ?e,
                    "failed to restart player"
                );
            }
        }

        if should_remove {
            player_manager
                .players
                .retain(|_, player| player.node_id != node_id);
        }

        reconnect_node(player_manager.lavalink.clone(), node_id);
    }

    let exec_time = init_time.elapsed();
    if exec_time > HYDROGEN_LAVALINK_EVENT_THRESHOLD {
        event!(
            Level::WARN,
            time = ?exec_time,
            "handling the Lavalink event took too long"
        );
    } else if !spammy_message {
        event!(
            Level::INFO,
            time = ?exec_time,
            "Lavalink event handled"
        );
    } else {
        event!(
            Level::DEBUG,
            time = ?exec_time,
            "Lavalink event handled"
        );
    }
}

/// Process the Lavalink data.
async fn process_data(message: &Message, player_manager: &PlayerManager) {
    if let Some(event) = message.as_event() {
        process_event(event, player_manager).await;
    }
}

/// Process the Lavalink event.
async fn process_event(event: &Event, player_manager: &PlayerManager) {
    match event {
        Event::TrackStart(track) => {
            if let Some(guild_id) = track.guild_id.parse::<u64>().ok().map(GuildId::new) {
                player_manager.update_message(guild_id).await;
            }
        }
        Event::TrackEnd(track) => {
            if track.reason.may_start_next() {
                if let Some(guild_id) = track.guild_id.parse::<u64>().ok().map(GuildId::new) {
                    if let Err(e) = player_manager.next_track(guild_id).await {
                        event!(
                            Level::ERROR,
                            error = %e,
                            guild_id = %guild_id,
                            "failed to play the next track"
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

/// Reconnect a Lavalink node, retrying until it connects.
pub fn reconnect_node(lavalink: Arc<Cluster>, node_id: usize) {
    event!(
        Level::DEBUG,
        node_id = node_id,
        "reconnecting to Lavalink..."
    );
    tokio::spawn(
        async move {
            sleep(Duration::from_secs(LAVALINK_RECONNECTION_DELAY)).await;
            while let Err(e) = lavalink.connect(node_id).await {
                event!(Level::WARN, error = %e, "failed to reconnect to Lavalink ");
                sleep(Duration::from_secs(LAVALINK_RECONNECTION_DELAY)).await;
            }
            event!(Level::INFO, "reconnected to Lavalink");
        }
        .instrument(span!(
            Level::TRACE,
            "lavalink_reconnection",
            node_id = node_id
        )),
    );
}
