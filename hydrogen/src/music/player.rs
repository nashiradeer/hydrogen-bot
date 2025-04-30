//! Player information and structures.

use std::fmt::{self, Display, Formatter};

use hydrolink::Track as LavalinkTrack;
use serenity::all::{ChannelId, GuildId, MessageId, ReactionType, UserId};
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
    /// Like [None], but automatically pauses after the track ends.
    AutoPause,
}

impl LoopMode {
    /// Get the next loop mode.
    pub fn next(&self) -> Self {
        match self {
            LoopMode::None => LoopMode::Single,
            LoopMode::Single => LoopMode::All,
            LoopMode::All => LoopMode::AutoPause,
            LoopMode::AutoPause => LoopMode::None,
        }
    }
}

impl Display for LoopMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LoopMode::None => write!(f, "➡️"),
            LoopMode::Single => write!(f, "🔂"),
            LoopMode::All => write!(f, "🔁"),
            LoopMode::AutoPause => write!(f, "↩️"),
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

impl Track {
    /// Create a new track.
    pub fn from_track(track: LavalinkTrack, requester: UserId) -> Self {
        Self {
            track: track.encoded,
            title: track.info.title,
            author: track.info.author,
            requester,
            duration: track.info.length,
            url: track.info.uri,
            thumbnail: track.info.artwork_url,
        }
    }
}

impl From<LavalinkTrack> for Track {
    fn from(track: LavalinkTrack) -> Self {
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

impl PlayResult {
    /// Merge the results of adding tracks and syncing the player.
    pub fn merge(add_queue_result: AddQueueResult, sync_result: SyncResult) -> Self {
        Self {
            track: sync_result.track,
            count: add_queue_result.count,
            playing: sync_result.playing,
            truncated: add_queue_result.truncated,
        }
    }
}

#[derive(Debug, Clone)]
/// Seek result information.
pub struct SeekResult {
    /// The position of the track in milliseconds.
    pub position: u64,
    /// The total duration of the track in milliseconds.
    pub total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Templates for player creation.
pub enum PlayerTemplate {
    /// Default player with pause disabled and no loop.
    Default,
    /// Player for playing music with single loop.
    Music,
    /// Player for playing a queue with all loop.
    Queue,
    /// Player for manual control with automatic pause.
    Manual,
    /// Player for RPG music with single loop and paused by default.
    Rpg,
}

impl PlayerTemplate {
    /// If the player should be paused by default.
    pub fn pause(&self) -> bool {
        matches!(self, Self::Manual | Self::Rpg)
    }

    /// The loop mode for the player.
    pub fn loop_mode(&self) -> LoopMode {
        match self {
            Self::Music | Self::Rpg => LoopMode::Single,
            Self::Queue => LoopMode::All,
            Self::Manual => LoopMode::AutoPause,
            _ => LoopMode::None,
        }
    }

    /// Convert the template into a player.
    pub fn into_player(self, node_id: usize, locale: &str, channel_id: ChannelId) -> Player {
        Player::new(node_id, locale, channel_id, self.loop_mode(), self.pause())
    }
}

impl Default for PlayerTemplate {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone)]
/// Result of fetching tracks.
pub struct FetchResult {
    /// The selected track.
    pub selected: Option<usize>,
    /// The tracks fetched.
    pub tracks: Vec<LavalinkTrack>,
}

#[derive(Debug, Clone)]
/// Result of adding tracks to the queue.
pub struct AddQueueResult {
    /// The track index that was selected to be played.
    pub selected: Option<usize>,
    /// The index of the first track added.
    pub first_track_index: usize,
    /// The amount of tracks that were added.
    pub count: usize,
    /// If the queue was truncated.
    pub truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Operation to add tracks to the queue.
pub enum AddQueueOperation {
    /// Add the track to the end of the queue.
    End,
    /// Add the track to next of the current track.
    Next,
}

impl Default for AddQueueOperation {
    fn default() -> Self {
        Self::End
    }
}

#[derive(Debug, Clone)]
/// Result of syncing the player.
pub struct SyncResult {
    /// The track that will be played.
    pub track: Option<Track>,
    /// If the player has started playing.
    pub playing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The mode to play the track.
pub enum PlayMode {
    /// Add the track to the end of the queue.
    AddToEnd,
    /// Add the track to the next of the current track.
    AddToNext,
    /// Play the track now.
    PlayNow,
}

impl Default for PlayMode {
    fn default() -> Self {
        Self::AddToEnd
    }
}

#[derive(Debug, Clone)]
pub struct PlayRequest<'a> {
    /// The track to play.
    pub music: &'a str,
    /// The requester of the track.
    pub requester: UserId,
    /// The guild ID of the player.
    pub guild_id: GuildId,
    /// The text channel of the player.
    pub text_channel: ChannelId,
    /// Locale for the player's messages.
    pub locale: &'a str,
    /// The player template to use.
    pub player_template: PlayerTemplate,
    /// The mode to play the track.
    pub play_mode: PlayMode,
}
