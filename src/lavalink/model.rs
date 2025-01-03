//! Models for the Lavalink REST API and WebSocket API.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
/// WebSocket message received from Lavalink server.
pub enum Message {
    #[serde(rename_all = "camelCase")]
    /// Dispatched when you successfully connect to the Lavalink node.
    Ready {
        /// Whether this session was resumed.
        resumed: bool,
        /// The Lavalink session id of this connection. Not to be confused with a Discord voice session id.
        session_id: String,
    },

    #[serde(rename_all = "camelCase")]
    /// Dispatched every x seconds with the latest player state.
    PlayerUpdate {
        /// The guild id of the player.
        guild_id: String,
        /// The player state.
        state: PlayerState,
    },

    #[serde(rename_all = "camelCase")]
    /// Dispatched when the node sends stats once per minute.
    Stats {
        /// The amount of players connected to the node.
        players: u32,
        /// The amount of players playing a track.
        playing_players: u32,
        /// The uptime of the node in milliseconds.
        uptime: u64,
        /// The memory stats of the node.
        memory: Memory,
        /// The cpu stats of the node.
        cpu: CPU,
        /// The frame stats of the node. [Option::None] if the node has no players or when retrieved via `/v4/stats`.
        frame_stats: Option<FrameStats>,
    },

    #[serde(rename_all = "camelCase")]
    /// Dispatched when player or voice events occur.
    Event(Event),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// State of the player.
pub struct PlayerState {
    /// Unix timestamp in milliseconds.
    pub time: u64,
    /// The position of the track in milliseconds.
    pub position: u64,
    /// Whether Lavalink is connected to the voice gateway.
    pub connected: bool,
    /// The ping of the node to the Discord voice server in milliseconds. (-1 if not connected)
    pub ping: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Memory statistics of the Lavalink node.
pub struct Memory {
    /// The amount of free memory in bytes.
    pub free: u64,
    /// The amount of used memory in bytes.
    pub used: u64,
    /// The amount of allocated memory in bytes.
    pub allocated: u64,
    /// The amount of reservable memory in bytes.
    pub reservable: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// CPU statistics of the Lavalink node.
pub struct CPU {
    /// The amount of cores the node has.
    pub cores: u16,
    /// The system load of the node.
    pub system_load: f32,
    /// The load of Lavalink on the node.
    pub lavalink_load: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Frame statistics of the Lavalink node.
pub struct FrameStats {
    /// The amount of frames sent to Discord.
    pub sent: u32,
    /// The amount of frames that were nulled.
    pub nulled: u32,
    /// The difference between sent frames and the expected amount of frames.
    pub deficit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
/// Websocket event received from Lavalink server.
pub enum Event {
    #[serde(rename_all = "camelCase", rename = "TrackStartEvent")]
    /// Dispatched when a track starts playing.
    TrackStart {
        /// The guild id.
        guild_id: String,
        /// The track that started playing.
        track: Track,
    },

    #[serde(rename_all = "camelCase", rename = "TrackEndEvent")]
    /// Dispatched when a track ends.
    TrackEnd {
        /// The guild id.
        guild_id: String,
        /// The track that ended playing.
        track: Track,
        /// The reason the track ended.
        reason: TrackEndReason,
    },

    #[serde(rename_all = "camelCase", rename = "TrackExceptionEvent")]
    /// Dispatched when a track throws an exception.
    TrackException {
        /// The guild id.
        guild_id: String,
        /// The track that threw the exception
        track: Track,
        /// The occurred exception.
        exception: Exception,
    },

    #[serde(rename_all = "camelCase", rename = "TrackStuckEvent")]
    /// Dispatched when a track gets stuck while playing.
    TrackStuck {
        /// The guild id.
        guild_id: String,
        /// The track that got stuck.
        track: Track,
        /// The threshold in milliseconds that was exceeded.
        threshold_ms: u32,
    },

    #[serde(rename_all = "camelCase", rename = "WebSocketClosedEvent")]
    /// Dispatched when an audio WebSocket (to Discord) is closed. This can happen for various reasons (normal and abnormal), e.g. when using an expired voice server update. 4xxx codes are usually bad. See the [Discord Docs](https://discord.com/developers/docs/topics/opcodes-and-status-codes#voice-voice-close-event-codes).
    WebSocketClosed {
        /// The guild id.
        guild_id: String,
        /// The [Discord close event code](https://discord.com/developers/docs/topics/opcodes-and-status-codes#voice-voice-close-event-codes).
        code: u32,
        /// The close reason.
        reason: String,
        /// Whether the connection was closed by Discord.
        by_remote: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Track object.
pub struct Track {
    /// The base64 encoded track data.
    pub encoded: String,

    /// Info about the track.
    pub info: TrackInfo,

    #[serde(default)]
    /// Additional track info provided by plugins.
    pub plugin_info: HashMap<String, Value>,

    #[serde(default)]
    /// Additional track data provided via the TODO endpoint.
    pub user_data: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Information about a track.
pub struct TrackInfo {
    /// The track identifier.
    pub identifier: String,
    /// Whether the track is seekable.
    pub is_seekable: bool,
    /// The track author.
    pub author: String,
    /// The track length in milliseconds.
    pub length: u64,
    /// Whether the track is a stream.
    pub is_stream: bool,
    /// The track position in milliseconds.
    pub position: u64,
    /// The track title.
    pub title: String,
    /// The track uri.
    pub uri: Option<String>,
    /// The track artwork url.
    pub artwork_url: Option<String>,
    /// The track [ISRC](https://en.wikipedia.org/wiki/International_Standard_Recording_Code).
    pub isrc: Option<String>,
    /// The track source name.
    pub source_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// Reasons a track has ended.
pub enum TrackEndReason {
    /// The track finished playing. (May Start Next? Yes)
    Finished,
    /// The track failed to load. (May Start Next? Yes)
    LoadFailed,
    /// The track was stopped. (May Start Next? No)
    Stopped,
    /// The track was replaced. (May Start Next? No)
    Replaced,
    /// The track was cleaned up. (May Start Next? No)
    Cleanup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents an exception thrown by the Lavalink node.
pub struct Exception {
    /// The message of the exception.
    pub message: Option<String>,
    /// The severity of the exception.
    pub severity: Severity,
    /// The cause of the exception.
    pub cause: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents the severity of an exception.
pub enum Severity {
    /// The cause is known and expected, indicates that there is nothing wrong with the library itself.
    Common,
    /// The cause might not be exactly known, but is possibly caused by outside factors. For example when an outside service responds in a format that we do not expect.
    Suspicous,
    /// The probable cause is an issue with the library or there is no way to tell what the cause might be. This is the default level and other levels are used in cases where the thrower has more in-depth knowledge about the error.
    Fault,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents an error from the REST API.
pub struct Error {
    /// The timestamp of the error in milliseconds since the Unix epoch.
    pub timestamp: i64,

    /// The HTTP status code.
    pub status: u32,

    /// The HTTP status code message.
    pub error: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The stack trace of the error when trace=true as query param has been sent.
    pub trace: Option<String>,

    /// The error message.
    pub message: String,

    /// The request path.
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
/// Represents a result from the REST API.
pub enum LavalinkResult<T> {
    /// Represents a successful result.
    Ok(T),
    /// Represents an error result.
    Err(Error),
}

impl<T> Into<Result<T, super::Error>> for LavalinkResult<T> {
    fn into(self) -> Result<T, super::Error> {
        match self {
            LavalinkResult::Ok(value) => Ok(value),
            LavalinkResult::Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Information about a playlist.
pub struct PlaylistInfo {
    /// The name of the playlist.
    pub name: String,
    /// The selected track of the playlist. (-1 if no track is selected)
    pub selected_track: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "loadType", content = "data")]
/// Represents the result of a load operation.
pub enum LoadResult {
    /// A track has been loaded.
    Track(Track),
    /// A playlist has been loaded.
    Playlist {
        /// The info of the playlist.
        info: PlaylistInfo,

        #[serde(default)]
        /// Addition playlist info provided by plugins.
        plugin_info: HashMap<String, Value>,

        /// The tracks of the playlist.
        tracks: Vec<Track>,
    },
    /// A search result has been loaded.
    Search(Vec<Track>),
    /// There has been no matches for your identifier.
    Empty,
    /// Loading has failed with an error.
    Error(Exception),
}

impl LoadResult {
    /// Check if the result is a track.
    pub fn is_track(&self) -> bool {
        matches!(self, Self::Track(_))
    }

    /// Check if the result is a playlist.
    pub fn is_playlist(&self) -> bool {
        matches!(self, Self::Playlist { .. })
    }

    /// Check if the result is a search.
    pub fn is_search(&self) -> bool {
        matches!(self, Self::Search(_))
    }

    /// Check if the result is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Check if the result is an error.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A player in the Lavalink node.
pub struct Player {
    /// The guild id of the player.
    pub guild_id: String,
    /// The currently playing track.
    pub track: Option<Track>,
    /// The volume of the player, range 0-1000, in percentage.
    pub volume: u16,
    /// Whether the player is paused.
    pub paused: bool,
    /// The state of the player.
    pub state: PlayerState,
    /// The voice state of the player.
    pub voice: VoiceState,
    /// The filters used by the player.
    pub filters: Filters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents a Discord voice state.
pub struct VoiceState {
    /// The Discord voice token to authenticate with.
    pub token: String,
    /// The Discord voice endpoint to connect to.
    pub endpoint: String,
    /// The Discord voice session id to authenticate with.
    pub session_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Configure the filters for the player.
pub struct Filters {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Adjusts the player volume from 0.0 to 5.0, where 1.0 is 100%. Values >1.0 may cause clipping.
    pub volume: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Adjusts 15 different bands.
    pub equalizer: Option<Vec<Equalizer>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Eliminates part of a band, usually targeting vocals.
    pub karaoke: Option<Karaoke>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Changes the speed, pitch, and rate.
    pub timescale: Option<Timescale>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Creates a shuddering effect, where the volume quickly oscillates.
    pub tremolo: Option<Tremolo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Creates a shuddering effect, where the pitch quickly oscillates.
    pub vibrato: Option<Vibrato>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Rotates the audio around the stereo channels/user headphones. (aka Audio Panning)
    pub rotation: Option<Rotation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Distorts the audio.
    pub distortion: Option<Distortion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Mixes both channels. (left and right)
    pub channel_mix: Option<ChannelMix>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Filters higher frequencies.
    pub low_pass: Option<LowPass>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Filter plugin configurations.
    pub plugin_filters: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// There are 15 bands (0-14) that can be changed. "gain" is the multiplier for the given band. The default value is 0. Valid values range from -0.25 to 1.0, where -0.25 means the given band is completely muted, and 0.25 means it is doubled. Modifying the gain could also change the volume of the output.
pub struct Equalizer {
    /// The band. (0 to 14)
    pub band: u8,
    /// The gain. (-0.25 to 1.0)
    pub gain: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Uses equalization to eliminate part of a band, usually targeting vocals.
pub struct Karaoke {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The level. (0 to 1.0 where 0.0 is no effect and 1.0 is full effect)
    pub level: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The mono level. (0 to 1.0 where 0.0 is no effect and 1.0 is full effect)
    pub mono_level: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The filter band. (in Hz)
    pub filter_band: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The filter width.
    pub filter_width: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Changes the speed, pitch, and rate. All default to 1.0.
pub struct Timescale {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The playback speed. 0.0 ≤ x
    pub speed: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The pitch. 0.0 ≤ x
    pub pitch: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The rate. 0.0 ≤ x
    pub rate: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Uses amplification to create a shuddering effect, where the volume quickly oscillates. Demo: https://en.wikipedia.org/wiki/File:Fuse_Electronics_Tremolo_MK-III_Quick_Demo.ogv
pub struct Tremolo {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The frequency. 0.0 < x
    pub frequency: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The tremolo depth. 0.0 < x ≤ 1.0
    pub depth: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Similar to tremolo. While tremolo oscillates the volume, vibrato oscillates the pitch.
pub struct Vibrato {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The frequency. 0.0 < x ≤ 14.0
    pub frequency: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The vibrato depth. 0.0 < x ≤ 1.0
    pub depth: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Rotates the sound around the stereo channels/user headphones (aka Audio Panning). It can produce an effect similar to https://youtu.be/QB9EB8mTKcc (without the reverb).
pub struct Rotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The frequency of the audio rotating around the listener in Hz. 0.2 is similar to the example video above.
    pub rotation_hz: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Distortion effect. It can generate some pretty unique audio effects.
pub struct Distortion {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The sin offset.
    pub sin_offset: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The sin scale.
    pub sin_scale: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The cos offset.
    pub cos_offset: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The cos scale.
    pub cos_scale: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The tan offset.
    pub tan_offset: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The tan scale.
    pub tan_scale: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The offset.
    pub offset: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The scale.
    pub scale: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Mixes both channels (left and right), with a configurable factor on how much each channel affects the other. With the defaults, both channels are kept independent of each other. Setting all factors to 0.5 means both channels get the same audio.
pub struct ChannelMix {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The left to left channel mix factor. (0.0 ≤ x ≤ 1.0)
    pub left_to_left: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The left to right channel mix factor. (0.0 ≤ x ≤ 1.0)
    pub left_to_right: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The right to left channel mix factor. (0.0 ≤ x ≤ 1.0)
    pub right_to_left: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The right to right channel mix factor. (0.0 ≤ x ≤ 1.0)
    pub right_to_right: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Higher frequencies get suppressed, while lower frequencies pass through this filter, thus the name low pass. Any smoothing values equal to or less than 1.0 will disable the filter.
pub struct LowPass {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The smoothing factor. (1.0 < x)
    pub smoothing: Option<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Update the player.
pub struct UpdatePlayer {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Specification for a new track to load, as well as user data to set.
    pub track: Option<UpdatePlayerTrack>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The track position in milliseconds.
    pub position: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ///The track end time in milliseconds (must be > 0). null resets this if it was set previously.
    pub end_time: Option<Option<u64>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The player volume, in percentage, from 0 to 1000.
    pub volume: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether the player is paused.
    pub paused: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The new filters to apply. This will override all previously applied filters.
    pub filters: Option<Filters>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Information required for connecting to Discord.
    pub voice: Option<VoiceState>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Update the player's track.
pub struct UpdatePlayerTrack {
    /// The base64 encoded track to play. [Option::None] stops the current track.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded: Option<Option<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The identifier of the track to play.
    pub identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ///	Additional track data to be sent back in the [Track].
    pub user_data: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request to update the session.
pub struct UpdateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether resuming is enabled for this session or not.
    pub resuming: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The timeout in seconds. (default is 60s)
    pub timeout: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Response to updating the session.
pub struct UpdateSessionResponse {
    /// Whether resuming is enabled for this session or not.
    pub resuming: bool,
    /// The timeout in seconds. (default is 60s)
    pub timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Information about the Lavalink server.
pub struct Info {
    /// The version of this Lavalink server.
    pub version: Version,
    /// The millisecond unix timestamp when this Lavalink jar was built.
    pub build_time: i64,
    /// The git information of this Lavalink server.
    pub git: Git,
    /// The JVM version this Lavalink server runs on.
    pub jvm: String,
    /// The Lavaplayer version being used by this server.
    pub lavaplayer: String,
    /// The enabled source managers for this server.
    pub source_managers: Vec<String>,
    /// The enabled filters for this server.
    pub filters: Vec<String>,
    /// The enabled plugins for this server.
    pub plugins: Vec<Plugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Information about the Lavalink server's version.
pub struct Version {
    /// The full version string of this Lavalink server.
    pub semver: String,
    /// The major version of this Lavalink server.
    pub major: u8,
    /// The minor version of this Lavalink server.
    pub minor: u8,
    /// The patch version of this Lavalink server.
    pub patch: u8,
    /// The pre-release version according to semver as a `.` separated list of identifiers.
    pub pre_release: Option<String>,
    /// The build metadata according to semver as a `.` separated list of identifiers.
    pub build: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Git information about the Lavalink server.
pub struct Git {
    /// The commit this Lavalink server was built on.
    pub commit: String,
    /// The branch this Lavalink server was built on.
    pub branch: String,
    /// The millisecond unix timestamp for when the commit was created.
    pub commit_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Information about a plugin in the Lavalink node.
pub struct Plugin {
    /// The name of the plugin.
    pub name: String,
    /// The version of the plugin.
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Object for the RoutePlanner API.
pub enum RoutePlanner {
    #[serde(rename = "RotatingIpRoutePlanner")]
    /// IP address used is switched on ban. Recommended for IPv4 blocks or IPv6 blocks smaller than a /64.
    Rotating {
        /// The ip block being used.
        ip_block: IPBlock,
        /// The failing addresses.
        failing_addresses: Vec<FailingAddress>,
        /// The number of rotations.
        rotate_index: String,
        /// The current offset in the block.
        ip_index: String,
        /// The current address being used.
        current_address: String,
    },

    #[serde(rename = "NanoIpRoutePlanner")]
    /// IP address used is switched on clock update. Use with at least 1 /64 IPv6 block.
    Nano {
        /// The ip block being used.
        ip_block: IPBlock,
        /// The failing addresses.
        failing_addresses: Vec<FailingAddress>,
        /// The current offset in the ip block.
        current_address_index: String,
    },

    #[serde(rename = "RotatingNanoIpRoutePlanner")]
    /// IP address used is switched on clock update, rotates to a different /64 block on ban. Use with at least 2x /64 IPv6 blocks.
    RotatingNano {
        /// The ip block being used.
        ip_block: IPBlock,
        /// The failing addresses.
        failing_addresses: Vec<FailingAddress>,
        /// The current offset in the ip block.
        current_address_index: String,
        /// The information in which /64 block ips are chosen. This number increases on each ban.
        block_index: String,
    },

    #[serde(rename = "BalancingIpRoutePlanner")]
    /// IP address used is selected at random per request. Recommended for larger IP blocks.
    Balancing {
        /// The ip block being used.
        ip_block: IPBlock,
        /// The failing addresses.
        failing_addresses: Vec<FailingAddress>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "size")]
/// Represents a IP block with size and type.
pub enum IPBlock {
    #[serde(rename = "Inet4Address")]
    /// The ipv4 block type.
    Inet4(String),

    #[serde(rename = "Inet6Address")]
    /// The ipv6 block type.
    Inet6(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents a failing address.
pub struct FailingAddress {
    /// The failing address.
    pub failing_address: String,
    /// The timestamp when the address failed.
    pub failing_timestamp: i64,
    /// The timestamp when the address failed as a pretty string.
    pub failing_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Unmark a route planner.
pub struct UnmarkRoutePlanner {
    /// The address to unmark.
    pub address: String,
}
