//! Player information and structures.

use std::fmt::{self, Display, Formatter};

use serenity::all::{ChannelId, MessageId, ReactionType, UserId};
use songbird::id::ChannelId as SongbirdChannelId;
use songbird::ConnectionInfo;
use tokio::task::JoinHandle;

use crate::lavalink::VoiceState;

#[derive(Debug)]
/// Player information.
pub struct Player {
    /// The text channel where the player is sending messages.
    pub channel_id: Option<ChannelId>,
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
            primary_queue: Vec::new(),
            _secondary_queue: None,
            currrent_track: 0,
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
            track: player.primary_queue.get(player.currrent_track).cloned(),
            node_id: player.node_id,
            loop_mode: player.loop_mode,
        }
    }
}

impl From<Player> for PlayerState {
    fn from(mut player: Player) -> Self {
        let track = if player.primary_queue.len() > player.currrent_track {
            Some(player.primary_queue.remove(player.currrent_track))
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
/// Player connection information.
pub struct PlayerConnection {
    /// The session ID of the Discord voice connection.
    pub session_id: Option<String>,
    /// The token for the Discord voice connection.
    pub token: Option<String>,
    /// The endpoint for the Discord voice connection.
    pub endpoint: Option<String>,
    /// The voice channel where the player is connected.
    pub channel_id: Option<SongbirdChannelId>,
}

impl PlayerConnection {
    /// Create a new player connection.
    pub fn _new(
        session_id: &str,
        token: &str,
        endpoint: &str,
        channel_id: SongbirdChannelId,
    ) -> Self {
        Self {
            session_id: Some(session_id.to_owned()),
            token: Some(token.to_owned()),
            endpoint: Some(endpoint.to_owned()),
            channel_id: Some(channel_id),
        }
    }

    /// Set the session ID for the player connection.
    pub fn set_session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_owned());
        self
    }

    /// Set the token for the player connection.
    pub fn set_token(mut self, token: &str) -> Self {
        self.token = Some(token.to_owned());
        self
    }

    /// Set the endpoint for the player connection.
    pub fn _set_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = Some(endpoint.to_owned());
        self
    }

    /// Set the channel ID for the player connection.
    pub fn set_channel_id(mut self, channel_id: SongbirdChannelId) -> Self {
        self.channel_id = Some(channel_id);
        self
    }

    /// Check if the player connection is ready.
    pub fn is_ready(&self) -> bool {
        self.session_id.is_some()
            && self.token.is_some()
            && self.endpoint.is_some()
            && self.channel_id.is_some()
    }

    /// Get and convert channel ID to Serenity's ChannelId.
    pub fn serenity_channel_id(&self) -> Option<ChannelId> {
        self.channel_id.map(|id| ChannelId::new(id.0.into()))
    }
}

impl From<&ConnectionInfo> for PlayerConnection {
    fn from(info: &ConnectionInfo) -> Self {
        Self {
            channel_id: info.channel_id,
            session_id: Some(info.session_id.clone()),
            token: Some(info.token.clone()),
            endpoint: Some(info.endpoint.clone()),
        }
    }
}

impl From<ConnectionInfo> for PlayerConnection {
    fn from(info: ConnectionInfo) -> Self {
        Self {
            channel_id: info.channel_id,
            session_id: Some(info.session_id),
            token: Some(info.token),
            endpoint: Some(info.endpoint),
        }
    }
}

impl TryInto<VoiceState> for PlayerConnection {
    type Error = Self;

    fn try_into(self) -> Result<VoiceState, Self> {
        if self.session_id.is_none() || self.token.is_none() || self.endpoint.is_none() {
            return Err(self);
        }

        Ok(VoiceState::new(
            &self.token.unwrap(),
            &self.endpoint.unwrap(),
            &self.session_id.unwrap(),
        ))
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
