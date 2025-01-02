//! Lavalink implementation to interact with multiple Lavalink nodes.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::StreamExt;
use tokio::sync::mpsc;

use super::{model::*, Lavalink, Result};

pub const LAVALINK_BUFFER_SIZE: usize = 8;

/// Manages multiple Lavalink nodes using a round-robin strategy and a multi-producer, single-consumer channel to receive messages.
pub struct Cluster {
    /// List of Lavalink nodes. Need to be immutable to avoid index changes.
    nodes: Arc<Vec<Lavalink>>,
    /// Sender to be used by the nodes to send messages.
    sender: mpsc::Sender<(usize, Option<Result<Message>>)>,
    /// Receiver to receive messages from the nodes.
    receiver: mpsc::Receiver<(usize, Option<Result<Message>>)>,
    /// Index for the round-robin strategy.
    index: AtomicUsize,
}

impl Cluster {
    /// Create a new Lavalink cluster.
    ///
    /// You can use [LAVALINK_BUFFER_SIZE] as the buffer size. (Recommended)
    pub async fn new(nodes: Arc<Vec<Lavalink>>, buffer_size: usize) -> Self {
        let (sender, receiver) = mpsc::channel(buffer_size);

        for index in 0..nodes.len() {
            let node_sender = sender.clone();
            let nodes_ref = nodes.clone();

            tokio::spawn(async move {
                let node = &nodes_ref[index];

                while let Some(message) = node.next().await {
                    if let Err(_) = node_sender.send((index, Some(message))).await {
                        break;
                    }
                }

                _ = node_sender.send((index, None)).await;
            });
        }

        Self {
            nodes,
            sender,
            receiver,
            index: AtomicUsize::new(0),
        }
    }

    /// Get the list of Lavalink nodes.
    pub fn nodes(&self) -> &Arc<Vec<Lavalink>> {
        &self.nodes
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

    /// Get the Lavaling node using [Cluster::next_index] as the index.
    pub fn node(&self) -> &Lavalink {
        &self.nodes[self.next_index()]
    }

    /// Receive a message from the nodes.
    pub async fn recv(&mut self) -> Option<(usize, Option<Result<Message>>)> {
        self.receiver.recv().await
    }
}
