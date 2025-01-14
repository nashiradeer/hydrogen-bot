use std::{sync::Arc, time::Duration};

use serenity::all::GuildId;
use tokio::time::sleep;
use tracing::{debug, error, warn};

use crate::{
    lavalink::{cluster::Cluster, Event, Message, TrackEndReason},
    utils::constants::LAVALINK_RECONNECTION_DELAY,
};

use super::PlayerManager;

/// Handle the Lavalink events.
pub fn handle_lavalink(player_manager: PlayerManager) {
    tokio::spawn(async move {
        while let Some((node_id, message)) = player_manager.lavalink.recv().await {
            if let Some(message) = message {
                match message {
                    Ok(data) => {
                        debug!(
                            "(music): received a message from Lavalink node {}: {:?}",
                            node_id, data
                        );
                        let player_manager = player_manager.clone();
                        tokio::spawn(async move {
                            process_data(data, &player_manager).await;
                        });
                    }
                    Err(e) => error!(
                        "(music): received an error from Lavalink node {}: {}",
                        node_id, e
                    ),
                }
            } else {
                warn!(
                    "(music): Lavalink node {} has disconnected, reconnecting in {} seconds",
                    node_id, LAVALINK_RECONNECTION_DELAY
                );

                let mut should_remove = false;

                for mut player in player_manager.players.iter_mut() {
                    if player.value().node_id == node_id {
                        if let Some(node_id) = player_manager.lavalink.search_connected_node().await
                        {
                            player.value_mut().node_id = node_id;
                        } else {
                            error!(
                                "(music): there's no available Lavalink node to migrate the players, all remaining players will be removed"
                            );
                            should_remove = true;
                            break;
                        }
                    }
                }

                if should_remove {
                    player_manager
                        .players
                        .retain(|_, player| player.node_id != node_id);
                }

                reconnect_node(player_manager.lavalink.clone(), node_id);
            }
        }
    });
}

/// Process the Lavalink data.
async fn process_data(message: Message, player_manager: &PlayerManager) {
    match message {
        Message::Event(event) => process_event(event, player_manager).await,
        _ => {}
    }
}

/// Process the Lavalink event.
async fn process_event(event: Event, player_manager: &PlayerManager) {
    match event {
        Event::TrackStart { guild_id, .. } => {
            if let Some(guild_id) = u64::from_str_radix(&guild_id, 10).ok().map(GuildId::new) {
                player_manager.update_message(guild_id).await;
            }
        }
        Event::TrackEnd {
            guild_id, reason, ..
        } => {
            if reason == TrackEndReason::Finished || reason == TrackEndReason::LoadFailed {
                if let Some(guild_id) = u64::from_str_radix(&guild_id, 10).ok().map(GuildId::new) {
                    if let Err(e) = player_manager.next_track(guild_id).await {
                        error!("failed to play the next track in guild {}: {}", guild_id, e);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Reconnect a Lavalink node, retrying until it connects.
pub fn reconnect_node(lavalink: Arc<Cluster>, node_id: usize) {
    tokio::spawn(async move {
        sleep(Duration::from_secs(LAVALINK_RECONNECTION_DELAY)).await;
        while let Err(e) = lavalink.connect(node_id).await {
            warn!(
                "(music): failed to reconnect to Lavalink node {}, retrying in {} seconds: {}",
                node_id, LAVALINK_RECONNECTION_DELAY, e
            );
            sleep(Duration::from_secs(LAVALINK_RECONNECTION_DELAY)).await;
        }
    });
}
