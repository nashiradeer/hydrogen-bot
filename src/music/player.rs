//! Player information and structures.

use std::fmt::{self, Display, Formatter};

use serenity::all::{ChannelId, MessageId, ReactionType, UserId};
use tokio::task::JoinHandle;

#[derive(Debug)]
/// Player information.
pub struct Player {
    /// The voice channel where the player is connected.
    pub voice_channel: ChannelId,
    /// The text channel where the player is connected.
    pub text_channel: Option<ChannelId>,
    /// The message ID of the player.
    pub message_id: Option<MessageId>,
    /// The queue of tracks.
    pub primary_queue: Vec<Track>,
    /// The secondary queue of tracks for PlayTogether.
    pub _secondary_queue: Option<Vec<Track>>,
    /// The current track being played.
    pub currrent_track: usize,
    /// The loop mode of the player.
    pub loop_mode: LoopMode,
    /// If the player is paused.
    pub paused: bool,
    /// The Lavalink node ID.
    pub node_id: usize,
    /// The translation locale for the player messages.
    pub locale: String,
    /// The session ID of the Discord voice connection.
    pub session_id: String,
    /// The token for the Discord voice connection.
    pub token: String,
    /// The endpoint for the Discord voice connection.
    pub endpoint: String,
    /// The handle for the player's destroy task.
    pub destroy_handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Loop mode for the player.
pub enum LoopMode {
    /// No loop.
    None,
    /// Loop the current track.
    Single,
    /// Loop the queue.
    All,
    /// Like [None], but autopausing the player.
    Autopause,
}

impl LoopMode {
    /// Get the next loop mode.
    pub fn next(&self) -> Self {
        match self {
            LoopMode::None => LoopMode::Single,
            LoopMode::Single => LoopMode::All,
            LoopMode::All => LoopMode::Autopause,
            LoopMode::Autopause => LoopMode::None,
        }
    }
}

impl Display for LoopMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LoopMode::None => write!(f, "➡️"),
            LoopMode::Single => write!(f, "🔂"),
            LoopMode::All => write!(f, "🔁"),
            LoopMode::Autopause => write!(f, "↩️"),
        }
    }
}

impl Into<ReactionType> for LoopMode {
    fn into(self) -> ReactionType {
        ReactionType::Unicode(self.to_string())
    }
}

impl Default for LoopMode {
    fn default() -> Self {
        LoopMode::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Track information.
pub struct Track {
    /// The track's identifier.
    pub track: String,
    /// The track's author.
    pub author: String,
    /// The track's title.
    pub title: String,
    /// Cache ID of the track's requester.
    pub requester: UserId,
    /// The track's duration.
    pub duration: u64,
    /// The track's URL for the user.
    pub url: Option<String>,
    /// The track's thumbnail.
    pub thumbnail: Option<String>,
}

impl From<crate::lavalink::Track> for Track {
    fn from(track: crate::lavalink::Track) -> Self {
        Self {
            track: track.encoded,
            title: track.info.title,
            author: track.info.author,
            requester: Default::default(),
            duration: track.info.length,
            url: track.info.uri,
            thumbnail: track.info.artwork_url,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Play result information.
pub struct PlayResult {
    /// The track that will be played.
    pub track: Option<Track>,
    /// The amount of tracks that will be played.
    pub count: usize,
    /// If the player is playing.
    pub playing: bool,
    /// If the queue was truncated.
    pub truncated: bool,
}
