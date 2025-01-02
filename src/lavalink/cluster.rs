//! Lavalink implementation to interact with multiple Lavalink nodes.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use futures::StreamExt;
use parking_lot::RwLock;
use tokio::{
    select,
    sync::{mpsc, Mutex as AsyncMutex, Notify},
};

use super::{
    model::*,
    utils::{connect, parse_message},
    Error, Rest, Result,
};

pub const LAVALINK_BUFFER_SIZE: usize = 8;

/// Manages multiple Lavalink nodes using a round-robin strategy and a multi-producer, single-consumer channel to receive messages.
#[derive(Debug)]
pub struct Cluster {
    /// List of Lavalink nodes. Need to be immutable to avoid index changes.
    nodes: Vec<Rest>,
    /// Sender to be used by the nodes to send messages.
    sender: mpsc::Sender<(usize, Option<Result<Message>>)>,
    /// Receiver to receive messages from the nodes.
    receiver: AsyncMutex<mpsc::Receiver<(usize, Option<Result<Message>>)>>,
    /// Notifier to notify the tasks when the connection needs to be closed.
    notifier: Arc<Notify>,
    /// Index for the round-robin strategy.
    index: AtomicUsize,
    /// The session ID from each node connection.
    session_id: Arc<RwLock<HashMap<usize, String>>>,
    /// The user ID to be used by the nodes.
    user_id: String,
}

impl Cluster {
    /// Create a new Lavalink cluster.
    pub async fn new(nodes: Vec<Rest>, user_id: &str) -> Self {
        let (sender, receiver) = mpsc::channel(1);

        Self {
            nodes,
            sender,
            receiver: AsyncMutex::new(receiver),
            index: AtomicUsize::new(0),
            notifier: Arc::new(Notify::new()),
            session_id: Arc::new(RwLock::new(HashMap::new())),
            user_id: user_id.to_owned(),
        }
    }

    /// Connect a node to the Lavalink server if it is not already connected.
    pub async fn connect(&self, index: usize) -> Result<()> {
        if self.is_connected(index) {
            return Err(Error::AlreadyConnected);
        }

        let sender = self.sender.clone();
        let notifier = self.notifier.clone();
        let node = &self.nodes[index];
        let session_id_storage = self.session_id.clone();
        let mut connection =
            connect(node.host(), node.password(), node.tls(), &self.user_id).await?;

        tokio::spawn(async move {
            loop {
                select! {
                    message = connection.next() => {
                        if let Some(msg) = message {
                            let data = parse_message(msg);

                            if let Ok(ref data) = data {
                                match data {
                                    Message::Ready {
                                        resumed: _,
                                        ref session_id,
                                    } => {
                                        session_id_storage.write().insert(index, session_id.clone());
                                        ()
                                    }
                                    _ => {}
                                };
                            }

                            if let Err(_) = sender.send((index, Some(data))).await {
                                break;
                            }
                        } else {
                            break;
                        }
                    },
                    _ = notifier.notified() => break,
                };
            }

            _ = sender.send((index, None)).await;
        });

        Ok(())
    }

    /// Get the list of Lavalink nodes.
    pub fn nodes(&self) -> &Vec<Rest> {
        &self.nodes
    }

    /// Get the list of connected nodes.
    pub fn connected_nodes(&self) -> Vec<usize> {
        self.session_id.read().keys().copied().collect()
    }

    /// Get the list of disconnected nodes.
    pub fn disconnected_nodes(&self) -> Vec<usize> {
        let connected = self.connected_nodes();
        (0..self.nodes.len())
            .filter(|x| !connected.contains(x))
            .collect()
    }

    /// Check if a node is connected.
    pub fn is_connected(&self, index: usize) -> bool {
        self.session_id.read().contains_key(&index)
    }

    /// Get the current index.
    pub fn current_index(&self) -> usize {
        self.index.load(Ordering::Relaxed)
    }

    /// Get the current index and increment it for the next call.
    pub fn next_index(&self) -> usize {
        self.index
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
                Some((x + 1) % self.nodes.len())
            })
            .unwrap()
    }

    /// Search for a connected node, returning the index if found or [None] if there is no connected node.
    ///
    /// This method uses the round-robin strategy to search for a connected node.
    pub fn search_connected_node(&self) -> Option<usize> {
        for _ in 0..self.nodes.len() {
            let index = self.next_index();
            if self.is_connected(index) {
                return Some(index);
            }
        }

        None
    }

    /// Get all players in the session.
    pub async fn get_players(&self, index: usize) -> Result<Vec<Player>> {
        self.nodes[index]
            .get_players(
                self.session_id
                    .read()
                    .get(&index)
                    .ok_or(Error::NoSessionId)?,
            )
            .await
    }

    /// Get the player in the session.
    pub async fn get_player(&self, index: usize, guild_id: &str) -> Result<Player> {
        self.nodes[index]
            .get_player(
                self.session_id
                    .read()
                    .get(&index)
                    .ok_or(Error::NoSessionId)?,
                guild_id,
            )
            .await
    }

    /// Update the player in the session.
    pub async fn update_player(
        &self,
        index: usize,
        guild_id: &str,
        player: &UpdatePlayer,
        no_replace: Option<bool>,
    ) -> Result<Player> {
        self.nodes[index]
            .update_player(
                self.session_id
                    .read()
                    .get(&index)
                    .ok_or(Error::NoSessionId)?,
                guild_id,
                player,
                no_replace,
            )
            .await
    }

    /// Destroy the player in the session.
    pub async fn destroy_player(&self, index: usize, guild_id: &str) -> Result<()> {
        self.nodes[index]
            .destroy_player(
                self.session_id
                    .read()
                    .get(&index)
                    .ok_or(Error::NoSessionId)?,
                guild_id,
            )
            .await
    }

    /// Update the session.
    pub async fn update_session(
        &self,
        index: usize,
        session: &UpdateSessionRequest,
    ) -> Result<UpdateSessionResponse> {
        self.nodes[index]
            .update_session(
                self.session_id
                    .read()
                    .get(&index)
                    .ok_or(Error::NoSessionId)?,
                session,
            )
            .await
    }

    /// Receive a message from the nodes.
    ///
    /// WARNING: This method locks the internal receiver mutex.
    pub async fn recv(&self) -> Option<(usize, Option<Result<Message>>)> {
        self.receiver.lock().await.recv().await
    }

    /// Close all connections to the Lavalink server.
    pub fn close(&self) {
        self.notifier.notify_waiters();
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        self.notifier.notify_waiters();
    }
}
