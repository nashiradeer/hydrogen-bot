//! Player information and structures.

use std::fmt::{self, Display, Formatter};

use serenity::all::{ChannelId, MessageId, ReactionType, UserId};
use tokio::task::JoinHandle;

#[derive(Debug)]
/// Player information.
pub struct Player {
    /// The text channel where the player is sending messages.
    pub channel_id: Option<ChannelId>,
    /// The message ID of the player.
    pub message_id: Option<MessageId>,
    /// The queue of tracks.
    pub queue: Vec<Track>,
    /// The current track being played.
    pub current_track: usize,
    /// The loop mode of the player.
    pub loop_mode: LoopMode,
    /// If the player is paused.
    pub paused: bool,
    /// The Lavalink node ID.
    pub node_id: usize,
    /// The translation locale for the player messages.
    pub locale: String,
    /// The handle for the player's destroy task.
    pub destroy_handle: Option<JoinHandle<()>>,
}

impl Player {
    /// Create a new player.
    pub fn new(
        node_id: usize,
        locale: &str,
        channel_id: ChannelId,
        loop_mode: LoopMode,
        paused: bool,
    ) -> Self {
        Self {
            channel_id: Some(channel_id),
            message_id: None,
            queue: Vec::new(),
            current_track: 0,
            loop_mode,
            paused,
            node_id,
            locale: locale.to_owned(),
            destroy_handle: None,
        }
    }

    /// Create a new player with the settings for the normal player.
    pub fn new_normal(node_id: usize, locale: &str, channel_id: ChannelId) -> Self {
        Self::new(node_id, locale, channel_id, LoopMode::None, false)
    }
}

#[derive(Debug, Clone)]
/// Represents the state of the player.
pub struct PlayerState {
    /// Whether the player is paused.
    pub paused: bool,
    /// Whether the player has a destroy handle.
    pub has_destroy_handle: bool,
    /// The channel ID of the player.
    pub text_channel: Option<ChannelId>,
    /// The message ID of the player.
    pub message_id: Option<MessageId>,
    /// The locale of the player.
    pub locale: String,
    /// The track currently playing.
    pub track: Option<Track>,
    /// The ID of the node used by this player.
    pub node_id: usize,
    /// The loop mode of the player.
    pub loop_mode: LoopMode,
}

impl From<&Player> for PlayerState {
    fn from(player: &Player) -> Self {
        Self {
            paused: player.paused,
            has_destroy_handle: player.destroy_handle.is_some(),
            text_channel: player.channel_id,
            message_id: player.message_id,
            locale: player.locale.clone(),
            track: player.queue.get(player.current_track).cloned(),
            node_id: player.node_id,
            loop_mode: player.loop_mode,
        }
    }
}

impl From<Player> for PlayerState {
    fn from(mut player: Player) -> Self {
        let track = if player.queue.len() > player.current_track {
            Some(player.queue.remove(player.current_track))
        } else {
            None
        };

        Self {
            paused: player.paused,
            has_destroy_handle: player.destroy_handle.is_some(),
            text_channel: player.channel_id,
            message_id: player.message_id,
            locale: player.locale,
            track,
            node_id: player.node_id,
            loop_mode: player.loop_mode,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// Loop mode for the player.
pub enum LoopMode {
    #[default]
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

impl From<LoopMode> for ReactionType {
    fn from(loop_mode: LoopMode) -> Self {
        ReactionType::Unicode(loop_mode.to_string())
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
/// Seek result information.
pub struct SeekResult {
    /// The track that was seeked.
    pub track: Track,
    /// The position of the track.
    pub position: u64,
    /// The total duration of the track.
    pub total: u64,
}
