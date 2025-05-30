//! Models for the Lavalink REST API and WebSocket API.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
/// WebSocket message received from Lavalink server.
pub enum Message {
    /// Dispatched when you successfully connect to the Lavalink node.
    Ready(Ready),
    /// Dispatched every x seconds with the latest player state.
    PlayerUpdate(PlayerUpdate),
    /// Dispatched when the node sends stats once per minute.
    Stats(Stats),
    /// Dispatched when player or voice events occur.
    Event(Box<Event>),
}

impl Message {
    /// Get the guild id of the message.
    pub fn guild_id(&self) -> Option<&String> {
        match self {
            Message::PlayerUpdate(player_update) => Some(&player_update.guild_id),
            Message::Event(event) => Some(event.guild_id()),
            _ => None,
        }
    }

    /// Get the kind of message.
    pub fn kind(&self) -> MessageKind {
        match self {
            Message::Ready(_) => MessageKind::Ready,
            Message::PlayerUpdate(_) => MessageKind::PlayerUpdate,
            Message::Stats(_) => MessageKind::Stats,
            Message::Event(_) => MessageKind::Event,
        }
    }

    /// Check if the message is ready.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    /// Check if the message is a player update.
    pub fn is_player_update(&self) -> bool {
        matches!(self, Self::PlayerUpdate(_))
    }

    /// Check if the message is stats.
    pub fn is_stats(&self) -> bool {
        matches!(self, Self::Stats(_))
    }

    /// Check if the message is an event.
    pub fn is_event(&self) -> bool {
        matches!(self, Self::Event(_))
    }

    /// Convert the message to ready.
    pub fn into_ready(self) -> Option<Ready> {
        match self {
            Message::Ready(ready) => Some(ready),
            _ => None,
        }
    }

    /// Convert the message to player update.
    pub fn into_player_update(self) -> Option<PlayerUpdate> {
        match self {
            Message::PlayerUpdate(player_update) => Some(player_update),
            _ => None,
        }
    }

    /// Convert the message to stats.
    pub fn into_stats(self) -> Option<Stats> {
        match self {
            Message::Stats(stats) => Some(stats),
            _ => None,
        }
    }

    /// Convert the message to event.
    pub fn into_event(self) -> Option<Event> {
        match self {
            Message::Event(event) => Some(*event),
            _ => None,
        }
    }

    /// Get the ready message.
    pub fn as_ready(&self) -> Option<&Ready> {
        match self {
            Message::Ready(ready) => Some(ready),
            _ => None,
        }
    }

    /// Get the player update message.
    pub fn as_player_update(&self) -> Option<&PlayerUpdate> {
        match self {
            Message::PlayerUpdate(player_update) => Some(player_update),
            _ => None,
        }
    }

    /// Get the stats message.
    pub fn as_stats(&self) -> Option<&Stats> {
        match self {
            Message::Stats(stats) => Some(stats),
            _ => None,
        }
    }

    /// Get the event message.
    pub fn as_event(&self) -> Option<&Event> {
        match self {
            Message::Event(event) => Some(event),
            _ => None,
        }
    }
}

impl From<Ready> for Message {
    fn from(ready: Ready) -> Self {
        Self::Ready(ready)
    }
}

impl From<PlayerUpdate> for Message {
    fn from(player_update: PlayerUpdate) -> Self {
        Self::PlayerUpdate(player_update)
    }
}

impl From<Stats> for Message {
    fn from(stats: Stats) -> Self {
        Self::Stats(stats)
    }
}

impl From<Event> for Message {
    fn from(event: Event) -> Self {
        Self::Event(Box::new(event))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kind of message received from the Lavalink server.
pub enum MessageKind {
    /// Dispatched when you successfully connect to the Lavalink node.
    Ready,
    /// Dispatched every x seconds with the latest player state.
    PlayerUpdate,
    /// Dispatched when the node sends stats once per minute.
    Stats,
    /// Dispatched when player or voice events occur.
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Dispatched when you successfully connect to the Lavalink node.
pub struct Ready {
    /// Whether this session was resumed.
    pub resumed: bool,
    /// The Lavalink session id of this connection. Not to be confused with a Discord voice session id.
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Dispatched every x seconds with the latest player state.
pub struct PlayerUpdate {
    /// The guild id of the player.
    pub guild_id: String,
    /// The player state.
    pub state: PlayerState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Dispatched when the node sends stats once per minute.
pub struct Stats {
    /// The amount of players connected to the node.
    pub players: u32,
    /// The amount of players playing a track.
    pub playing_players: u32,
    /// The uptime of the node in milliseconds.
    pub uptime: u64,
    /// The memory stats of the node.
    pub memory: Memory,
    /// The cpu stats of the node.
    pub cpu: Cpu,
    /// The frame stats of the node. [None] if the node has no players or when retrieved via `/v4/stats`.
    pub frame_stats: Option<FrameStats>,
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
pub struct Cpu {
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
    pub deficit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
/// Websocket event received from Lavalink server.
pub enum Event {
    #[serde(rename = "TrackStartEvent")]
    /// Dispatched when a track starts playing.
    TrackStart(TrackStartEvent),

    #[serde(rename = "TrackEndEvent")]
    /// Dispatched when a track ends.
    TrackEnd(TrackEndEvent),

    #[serde(rename = "TrackExceptionEvent")]
    /// Dispatched when a track throws an exception.
    TrackException(TrackExceptionEvent),

    #[serde(rename = "TrackStuckEvent")]
    /// Dispatched when a track gets stuck while playing.
    TrackStuck(TrackStuckEvent),

    #[serde(rename = "WebSocketClosedEvent")]
    /// Dispatched when an audio WebSocket (to Discord) is closed. This can happen for various reasons (normal and abnormal), e.g. when using an expired voice server update. 4xxx codes are usually bad. See the [Discord Docs](https://discord.com/developers/docs/topics/opcodes-and-status-codes#voice-voice-close-event-codes).
    WebSocketClosed(WebSocketClosedEvent),
}

impl Event {
    /// Get the kind of event.
    pub fn kind(&self) -> EventKind {
        match self {
            Event::TrackStart(_) => EventKind::TrackStart,
            Event::TrackEnd(_) => EventKind::TrackEnd,
            Event::TrackException(_) => EventKind::TrackException,
            Event::TrackStuck(_) => EventKind::TrackStuck,
            Event::WebSocketClosed(_) => EventKind::WebSocketClosed,
        }
    }

    /// Get the guild id of the event.
    pub fn guild_id(&self) -> &String {
        match self {
            Event::TrackStart(track_start) => &track_start.guild_id,
            Event::TrackEnd(track_end) => &track_end.guild_id,
            Event::TrackException(track_exception) => &track_exception.guild_id,
            Event::TrackStuck(track_stuck) => &track_stuck.guild_id,
            Event::WebSocketClosed(websocket_closed) => &websocket_closed.guild_id,
        }
    }

    /// Get the track of the event.
    pub fn track(&self) -> Option<&Track> {
        match self {
            Event::TrackStart(track_start) => Some(&track_start.track),
            Event::TrackEnd(track_end) => Some(&track_end.track),
            Event::TrackException(track_exception) => Some(&track_exception.track),
            Event::TrackStuck(track_stuck) => Some(&track_stuck.track),
            _ => None,
        }
    }

    /// Check if the event is a track start.
    pub fn is_track_start(&self) -> bool {
        matches!(self, Self::TrackStart(_))
    }

    /// Check if the event is a track end.
    pub fn is_track_end(&self) -> bool {
        matches!(self, Self::TrackEnd(_))
    }

    /// Check if the event is a track exception.
    pub fn is_track_exception(&self) -> bool {
        matches!(self, Self::TrackException(_))
    }

    /// Check if the event is a track stuck.
    pub fn is_track_stuck(&self) -> bool {
        matches!(self, Self::TrackStuck(_))
    }

    /// Check if the event is a websocket closed.
    pub fn is_websocket_closed(&self) -> bool {
        matches!(self, Self::WebSocketClosed(_))
    }

    /// Convert the event to track start.
    pub fn into_track_start(self) -> Option<TrackStartEvent> {
        match self {
            Event::TrackStart(track_start) => Some(track_start),
            _ => None,
        }
    }

    /// Convert the event to track end.
    pub fn into_track_end(self) -> Option<TrackEndEvent> {
        match self {
            Event::TrackEnd(track_end) => Some(track_end),
            _ => None,
        }
    }

    /// Convert the event to track exception.
    pub fn into_track_exception(self) -> Option<TrackExceptionEvent> {
        match self {
            Event::TrackException(track_exception) => Some(track_exception),
            _ => None,
        }
    }

    /// Convert the event to track stuck.
    pub fn into_track_stuck(self) -> Option<TrackStuckEvent> {
        match self {
            Event::TrackStuck(track_stuck) => Some(track_stuck),
            _ => None,
        }
    }

    /// Convert the event to websocket closed.
    pub fn into_websocket_closed(self) -> Option<WebSocketClosedEvent> {
        match self {
            Event::WebSocketClosed(websocket_closed) => Some(websocket_closed),
            _ => None,
        }
    }

    /// Get the track start event.
    pub fn as_track_start(&self) -> Option<&TrackStartEvent> {
        match self {
            Event::TrackStart(track_start) => Some(track_start),
            _ => None,
        }
    }

    /// Get the track end event.
    pub fn as_track_end(&self) -> Option<&TrackEndEvent> {
        match self {
            Event::TrackEnd(track_end) => Some(track_end),
            _ => None,
        }
    }

    /// Get the track exception event.
    pub fn as_track_exception(&self) -> Option<&TrackExceptionEvent> {
        match self {
            Event::TrackException(track_exception) => Some(track_exception),
            _ => None,
        }
    }

    /// Get the track stuck event.
    pub fn as_track_stuck(&self) -> Option<&TrackStuckEvent> {
        match self {
            Event::TrackStuck(track_stuck) => Some(track_stuck),
            _ => None,
        }
    }

    /// Get the websocket closed event.
    pub fn as_websocket_closed(&self) -> Option<&WebSocketClosedEvent> {
        match self {
            Event::WebSocketClosed(websocket_closed) => Some(websocket_closed),
            _ => None,
        }
    }
}

impl From<TrackStartEvent> for Event {
    fn from(track_start: TrackStartEvent) -> Self {
        Self::TrackStart(track_start)
    }
}

impl From<TrackEndEvent> for Event {
    fn from(track_end: TrackEndEvent) -> Self {
        Self::TrackEnd(track_end)
    }
}

impl From<TrackExceptionEvent> for Event {
    fn from(track_exception: TrackExceptionEvent) -> Self {
        Self::TrackException(track_exception)
    }
}

impl From<TrackStuckEvent> for Event {
    fn from(track_stuck: TrackStuckEvent) -> Self {
        Self::TrackStuck(track_stuck)
    }
}

impl From<WebSocketClosedEvent> for Event {
    fn from(websocket_closed: WebSocketClosedEvent) -> Self {
        Self::WebSocketClosed(websocket_closed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kind of event received from the Lavalink server.
pub enum EventKind {
    /// Dispatched when a track starts playing.
    TrackStart,
    /// Dispatched when a track ends.
    TrackEnd,
    /// Dispatched when a track throws an exception.
    TrackException,
    /// Dispatched when a track gets stuck while playing.
    TrackStuck,
    /// Dispatched when an audio WebSocket (to Discord) is closed.
    WebSocketClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Dispatched when a track starts playing.
pub struct TrackStartEvent {
    /// The guild id.
    pub guild_id: String,
    /// The track that started playing.
    pub track: Track,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Dispatched when a track ends.
pub struct TrackEndEvent {
    /// The guild id.
    pub guild_id: String,
    /// The track that ended playing.
    pub track: Track,
    /// The reason the track ended.
    pub reason: TrackEndReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Dispatched when a track throws an exception.
pub struct TrackExceptionEvent {
    /// The guild id.
    pub guild_id: String,
    /// The track that threw the exception
    pub track: Track,
    /// The occurred exception.
    pub exception: Exception,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Dispatched when a track gets stuck while playing.
pub struct TrackStuckEvent {
    /// The guild id.
    pub guild_id: String,
    /// The track that got stuck.
    pub track: Track,
    /// The threshold in milliseconds that was exceeded.
    pub threshold_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Dispatched when an audio WebSocket (to Discord) is closed. This can happen for various reasons (normal and abnormal), e.g. when using an expired voice server update. 4xxx codes are usually bad. See the [Discord Docs](https://discord.com/developers/docs/topics/opcodes-and-status-codes#voice-voice-close-event-codes).
pub struct WebSocketClosedEvent {
    /// The guild id.
    pub guild_id: String,
    /// The [Discord close event code](https://discord.com/developers/docs/topics/opcodes-and-status-codes#voice-voice-close-event-codes).
    pub code: u32,
    /// The close reason.
    pub reason: String,
    /// Whether the connection was closed by Discord.
    pub by_remote: bool,
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
    /// Additional track data provided via the [super::Rest::update_player] endpoint.
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl TrackEndReason {
    /// Check if the next track should start.
    pub fn may_start_next(&self) -> bool {
        matches!(self, Self::Finished | Self::LoadFailed)
    }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// Represents the severity of an exception.
pub enum Severity {
    /// The cause is known and expected, indicates that there is nothing wrong with the library itself.
    Common,
    /// The cause might not be exactly known, but is possibly caused by outside factors. For example when an outside service responds in a format that we do not expect.
    Suspicious,
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
/// A serializable enum containing the expected response or an error from the REST API.
///
/// This is used to parse the response from the Lavalink REST API avoiding needing to parse the response twice.
pub enum ApiResponse<T> {
    /// Expected response from the REST API.
    Ok(T),
    /// Error response from the REST API.
    Err(Error),
}

impl<T> From<ApiResponse<T>> for Result<T, super::Error> {
    fn from(result: ApiResponse<T>) -> Self {
        match result {
            ApiResponse::Ok(value) => Ok(value),
            ApiResponse::Err(err) => Err(err.into()),
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
    Track(Box<Track>),
    /// A playlist has been loaded.
    Playlist(LoadResultPlaylist),
    /// A search result has been loaded.
    Search(Vec<Track>),
    /// There has been no matches for your identifier.
    Empty,
    /// Loading has failed with an error.
    Error(Exception),
}

impl LoadResult {
    /// Get the kind of load result.
    pub fn kind(&self) -> LoadResultKind {
        match self {
            LoadResult::Track(_) => LoadResultKind::Track,
            LoadResult::Playlist(_) => LoadResultKind::Playlist,
            LoadResult::Search(_) => LoadResultKind::Search,
            LoadResult::Empty => LoadResultKind::Empty,
            LoadResult::Error(_) => LoadResultKind::Error,
        }
    }

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

    /// Convert the result to a track.
    pub fn into_track(self) -> Option<Track> {
        match self {
            LoadResult::Track(track) => Some(*track),
            _ => None,
        }
    }

    /// Convert the result to a playlist.
    pub fn into_playlist(self) -> Option<LoadResultPlaylist> {
        match self {
            LoadResult::Playlist(playlist) => Some(playlist),
            _ => None,
        }
    }

    /// Convert the result to a search.
    pub fn into_search(self) -> Option<Vec<Track>> {
        match self {
            LoadResult::Search(tracks) => Some(tracks),
            _ => None,
        }
    }

    /// Convert the result to an error.
    pub fn into_error(self) -> Option<Exception> {
        match self {
            LoadResult::Error(exception) => Some(exception),
            _ => None,
        }
    }

    /// Get the track result.
    pub fn as_track(&self) -> Option<&Track> {
        match self {
            LoadResult::Track(track) => Some(track),
            _ => None,
        }
    }

    /// Get the playlist result.
    pub fn as_playlist(&self) -> Option<&LoadResultPlaylist> {
        match self {
            LoadResult::Playlist(playlist) => Some(playlist),
            _ => None,
        }
    }

    /// Get the search result.
    pub fn as_search(&self) -> Option<&Vec<Track>> {
        match self {
            LoadResult::Search(tracks) => Some(tracks),
            _ => None,
        }
    }

    /// Get the error result.
    pub fn as_error(&self) -> Option<&Exception> {
        match self {
            LoadResult::Error(exception) => Some(exception),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kind of load result.
pub enum LoadResultKind {
    /// A track has been loaded.
    Track,
    /// A playlist has been loaded.
    Playlist,
    /// A search result has been loaded.
    Search,
    /// There has been no matches for your identifier.
    Empty,
    /// Loading has failed with an error.
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents the result of a load playlist operation.
pub struct LoadResultPlaylist {
    /// The info of the playlist.
    pub info: PlaylistInfo,

    #[serde(default)]
    /// Addition playlist info provided by plugins.
    pub plugin_info: HashMap<String, Value>,

    /// The tracks of the playlist.
    pub tracks: Vec<Track>,
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

impl VoiceState {
    /// Create a new voice state.
    pub fn new(token: &str, endpoint: &str, session_id: &str) -> Self {
        Self {
            token: token.to_owned(),
            endpoint: endpoint.to_owned(),
            session_id: session_id.to_owned(),
        }
    }
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

impl Filters {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// There are 15 bands (0-14) that can be changed. "gain" is the multiplier for the given band. The default value is 0. Valid values range from -0.25 to 1.0, where -0.25 means the given band is completely muted, and 0.25 means it is doubled. Modifying the gain could also change the volume of the output.
pub struct Equalizer {
    /// The band. (0 to 14)
    pub band: u8,
    /// The gain. (-0.25 to 1.0)
    pub gain: f32,
}

impl Equalizer {
    /// Create a new equalizer.
    pub fn new(band: u8, gain: f32) -> Self {
        Self { band, gain }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl Karaoke {
    /// Create a new karaoke.
    pub fn new(level: f32, mono_level: f32, filter_band: f32, filter_width: f32) -> Self {
        Self {
            level: Some(level),
            mono_level: Some(mono_level),
            filter_band: Some(filter_band),
            filter_width: Some(filter_width),
        }
    }

    /// Set the level.
    pub fn set_level(mut self, level: f32) -> Self {
        self.level = Some(level);
        self
    }

    /// Set the mono level.
    pub fn set_mono_level(mut self, mono_level: f32) -> Self {
        self.mono_level = Some(mono_level);
        self
    }

    /// Set the filter band.
    pub fn set_filter_band(mut self, filter_band: f32) -> Self {
        self.filter_band = Some(filter_band);
        self
    }

    /// Set the filter width.
    pub fn set_filter_width(mut self, filter_width: f32) -> Self {
        self.filter_width = Some(filter_width);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl Timescale {
    /// Create a new timescale.
    pub fn new(speed: f32, pitch: f32, rate: f32) -> Self {
        Self {
            speed: Some(speed),
            pitch: Some(pitch),
            rate: Some(rate),
        }
    }

    /// Set the speed.
    pub fn set_speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Set the pitch.
    pub fn set_pitch(mut self, pitch: f32) -> Self {
        self.pitch = Some(pitch);
        self
    }

    /// Set the rate.
    pub fn set_rate(mut self, rate: f32) -> Self {
        self.rate = Some(rate);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl Tremolo {
    /// Create a new tremolo.
    pub fn new(frequency: f32, depth: f32) -> Self {
        Self {
            frequency: Some(frequency),
            depth: Some(depth),
        }
    }

    /// Set the frequency.
    pub fn set_frequency(mut self, frequency: f32) -> Self {
        self.frequency = Some(frequency);
        self
    }

    /// Set the depth.
    pub fn set_depth(mut self, depth: f32) -> Self {
        self.depth = Some(depth);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl Vibrato {
    /// Create a new vibrato.
    pub fn new(frequency: f32, depth: f32) -> Self {
        Self {
            frequency: Some(frequency),
            depth: Some(depth),
        }
    }

    /// Set the frequency.
    pub fn set_frequency(mut self, frequency: f32) -> Self {
        self.frequency = Some(frequency);
        self
    }

    /// Set the depth.
    pub fn set_depth(mut self, depth: f32) -> Self {
        self.depth = Some(depth);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Rotates the sound around the stereo channels/user headphones (aka Audio Panning). It can produce an effect similar to https://youtu.be/QB9EB8mTKcc (without the reverb).
pub struct Rotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The frequency of the audio rotating around the listener in Hz. 0.2 is similar to the example video above.
    pub rotation_hz: Option<f32>,
}

impl Rotation {
    /// Create a new rotation.
    pub fn new(rotation_hz: f32) -> Self {
        Self {
            rotation_hz: Some(rotation_hz),
        }
    }

    /// Set the rotation hz.
    pub fn set_rotation_hz(mut self, rotation_hz: f32) -> Self {
        self.rotation_hz = Some(rotation_hz);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl Distortion {
    /// Set the sin offset.
    pub fn set_sin_offset(mut self, sin_offset: f32) -> Self {
        self.sin_offset = Some(sin_offset);
        self
    }

    /// Set the sin scale.
    pub fn set_sin_scale(mut self, sin_scale: f32) -> Self {
        self.sin_scale = Some(sin_scale);
        self
    }

    /// Set the cos offset.
    pub fn set_cos_offset(mut self, cos_offset: f32) -> Self {
        self.cos_offset = Some(cos_offset);
        self
    }

    /// Set the cos scale.
    pub fn set_cos_scale(mut self, cos_scale: f32) -> Self {
        self.cos_scale = Some(cos_scale);
        self
    }

    /// Set the tan offset.
    pub fn set_tan_offset(mut self, tan_offset: f32) -> Self {
        self.tan_offset = Some(tan_offset);
        self
    }

    /// Set the tan scale.
    pub fn set_tan_scale(mut self, tan_scale: f32) -> Self {
        self.tan_scale = Some(tan_scale);
        self
    }

    /// Set the offset.
    pub fn set_offset(mut self, offset: f32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set the scale.
    pub fn set_scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl ChannelMix {
    /// Create a new channel mix.
    pub fn new(
        left_to_left: f32,
        left_to_right: f32,
        right_to_left: f32,
        right_to_right: f32,
    ) -> Self {
        Self {
            left_to_left: Some(left_to_left),
            left_to_right: Some(left_to_right),
            right_to_left: Some(right_to_left),
            right_to_right: Some(right_to_right),
        }
    }

    /// Set the left to left channel mix factor.
    pub fn set_left_to_left(mut self, left_to_left: f32) -> Self {
        self.left_to_left = Some(left_to_left);
        self
    }

    /// Set the left to right channel mix factor.
    pub fn set_left_to_right(mut self, left_to_right: f32) -> Self {
        self.left_to_right = Some(left_to_right);
        self
    }

    /// Set the right to left channel mix factor.
    pub fn set_right_to_left(mut self, right_to_left: f32) -> Self {
        self.right_to_left = Some(right_to_left);
        self
    }

    /// Set the right to right channel mix factor.
    pub fn set_right_to_right(mut self, right_to_right: f32) -> Self {
        self.right_to_right = Some(right_to_right);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Higher frequencies get suppressed, while lower frequencies pass through this filter, thus the name low pass. Any smoothing values equal to or less than 1.0 will disable the filter.
pub struct LowPass {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The smoothing factor. (1.0 < x)
    pub smoothing: Option<f32>,
}

impl LowPass {
    /// Create a new low pass.
    pub fn new(smoothing: f32) -> Self {
        Self {
            smoothing: Some(smoothing),
        }
    }

    /// Set the smoothing factor.
    pub fn set_smoothing(mut self, smoothing: f32) -> Self {
        self.smoothing = Some(smoothing);
        self
    }
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
    pub position: Option<u64>,

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

impl UpdatePlayer {
    /// Set the track.
    pub fn set_track(mut self, track: UpdatePlayerTrack) -> Self {
        self.track = Some(track);
        self
    }

    /// Set the position.
    pub fn set_position(mut self, position: u64) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the end time.
    pub fn set_end_time(mut self, end_time: u64) -> Self {
        self.end_time = Some(Some(end_time));
        self
    }

    /// Reset the end time.
    pub fn reset_end_time(mut self) -> Self {
        self.end_time = Some(None);
        self
    }

    /// Set the volume.
    pub fn set_volume(mut self, volume: u16) -> Self {
        self.volume = Some(volume);
        self
    }

    /// Set the paused.
    pub fn set_paused(mut self, paused: bool) -> Self {
        self.paused = Some(paused);
        self
    }

    /// Set the filters.
    pub fn set_filters(mut self, filters: Filters) -> Self {
        self.filters = Some(filters);
        self
    }

    /// Set the voice state.
    pub fn set_voice(mut self, voice: VoiceState) -> Self {
        self.voice = Some(voice);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Update the player's track.
pub struct UpdatePlayerTrack {
    /// The base64 encoded track to play. `Some(None)` stops the current track.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded: Option<Option<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The identifier of the track to play.
    pub identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Additional track data to be sent back in the [Track].
    pub user_data: Option<HashMap<String, Value>>,
}

impl UpdatePlayerTrack {
    /// Set the encoded track.
    pub fn set_encoded(mut self, encoded: &str) -> Self {
        self.encoded = Some(Some(encoded.to_owned()));
        self.identifier = None;
        self
    }

    /// Set the identifier.
    pub fn set_identifier(mut self, identifier: &str) -> Self {
        self.identifier = Some(identifier.to_owned());
        self.encoded = None;
        self
    }

    /// Set the stop player.
    pub fn set_stop_player(mut self) -> Self {
        self.encoded = Some(None);
        self.identifier = None;
        self
    }

    /// Set the user data.
    pub fn set_user_data(mut self, user_data: HashMap<String, Value>) -> Self {
        self.user_data = Some(user_data);
        self
    }
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

impl UpdateSessionRequest {
    /// Create a new update session request.
    pub fn new(resuming: bool, timeout: u32) -> Self {
        Self {
            resuming: Some(resuming),
            timeout: Some(timeout),
        }
    }

    /// Set whether resuming is enabled for this session or not.
    pub fn set_resuming(mut self, resuming: bool) -> Self {
        self.resuming = Some(resuming);
        self
    }

    /// Set the timeout in seconds.
    pub fn set_timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout);
        self
    }
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
    Rotating(RotatingIpRoutePlanner),

    #[serde(rename = "NanoIpRoutePlanner")]
    /// IP address used is switched on clock update. Use with at least 1 /64 IPv6 block.
    Nano(NanoIpRoutePlanner),

    #[serde(rename = "RotatingNanoIpRoutePlanner")]
    /// IP address used is switched on clock update, rotates to a different /64 block on ban. Use with at least 2x /64 IPv6 blocks.
    RotatingNano(RotatingNanoIpRoutePlanner),

    #[serde(rename = "BalancingIpRoutePlanner")]
    /// IP address used is selected at random per request. Recommended for larger IP blocks.
    Balancing(BalancingIpRoutePlanner),
}

impl RoutePlanner {
    /// Get the ip block being used.
    pub fn ip_block(&self) -> &IPBlock {
        match self {
            RoutePlanner::Rotating(route_planner) => &route_planner.ip_block,
            RoutePlanner::Nano(route_planner) => &route_planner.ip_block,
            RoutePlanner::RotatingNano(route_planner) => &route_planner.ip_block,
            RoutePlanner::Balancing(route_planner) => &route_planner.ip_block,
        }
    }

    /// Get the failing addresses.
    pub fn failing_addresses(&self) -> &Vec<FailingAddress> {
        match self {
            RoutePlanner::Rotating(route_planner) => &route_planner.failing_addresses,
            RoutePlanner::Nano(route_planner) => &route_planner.failing_addresses,
            RoutePlanner::RotatingNano(route_planner) => &route_planner.failing_addresses,
            RoutePlanner::Balancing(route_planner) => &route_planner.failing_addresses,
        }
    }

    /// Get the current address being used.
    pub fn current_address_index(&self) -> Option<&String> {
        match self {
            RoutePlanner::Nano(route_planner) => Some(&route_planner.current_address_index),
            RoutePlanner::RotatingNano(route_planner) => Some(&route_planner.current_address_index),
            _ => None,
        }
    }

    /// Get the kind of route planner.
    pub fn kind(&self) -> RoutePlannerKind {
        match self {
            RoutePlanner::Rotating(_) => RoutePlannerKind::Rotating,
            RoutePlanner::Nano(_) => RoutePlannerKind::Nano,
            RoutePlanner::RotatingNano(_) => RoutePlannerKind::RotatingNano,
            RoutePlanner::Balancing(_) => RoutePlannerKind::Balancing,
        }
    }

    /// Check if the route planner is rotating.
    pub fn is_rotating(&self) -> bool {
        matches!(self, Self::Rotating(_))
    }

    /// Check if the route planner is nano.
    pub fn is_nano(&self) -> bool {
        matches!(self, Self::Nano(_))
    }

    /// Check if the route planner is rotating nano.
    pub fn is_rotating_nano(&self) -> bool {
        matches!(self, Self::RotatingNano(_))
    }

    /// Check if the route planner is balancing.
    pub fn is_balancing(&self) -> bool {
        matches!(self, Self::Balancing(_))
    }

    /// Convert the route planner to rotating.
    pub fn into_rotating(self) -> Option<RotatingIpRoutePlanner> {
        match self {
            RoutePlanner::Rotating(rotating) => Some(rotating),
            _ => None,
        }
    }

    /// Convert the route planner to nano.
    pub fn into_nano(self) -> Option<NanoIpRoutePlanner> {
        match self {
            RoutePlanner::Nano(nano) => Some(nano),
            _ => None,
        }
    }

    /// Convert the route planner to rotating nano.
    pub fn into_rotating_nano(self) -> Option<RotatingNanoIpRoutePlanner> {
        match self {
            RoutePlanner::RotatingNano(rotating_nano) => Some(rotating_nano),
            _ => None,
        }
    }

    /// Convert the route planner to balancing.
    pub fn into_balancing(self) -> Option<BalancingIpRoutePlanner> {
        match self {
            RoutePlanner::Balancing(balancing) => Some(balancing),
            _ => None,
        }
    }

    /// Get the rotating route planner.
    pub fn as_rotating(&self) -> Option<&RotatingIpRoutePlanner> {
        match self {
            RoutePlanner::Rotating(rotating) => Some(rotating),
            _ => None,
        }
    }

    /// Get the nano route planner.
    pub fn as_nano(&self) -> Option<&NanoIpRoutePlanner> {
        match self {
            RoutePlanner::Nano(nano) => Some(nano),
            _ => None,
        }
    }

    /// Get the rotating nano route planner.
    pub fn as_rotating_nano(&self) -> Option<&RotatingNanoIpRoutePlanner> {
        match self {
            RoutePlanner::RotatingNano(rotating_nano) => Some(rotating_nano),
            _ => None,
        }
    }

    /// Get the balancing route planner.
    pub fn as_balancing(&self) -> Option<&BalancingIpRoutePlanner> {
        match self {
            RoutePlanner::Balancing(balancing) => Some(balancing),
            _ => None,
        }
    }
}

impl From<RotatingIpRoutePlanner> for RoutePlanner {
    fn from(route_planner: RotatingIpRoutePlanner) -> Self {
        RoutePlanner::Rotating(route_planner)
    }
}

impl From<NanoIpRoutePlanner> for RoutePlanner {
    fn from(route_planner: NanoIpRoutePlanner) -> Self {
        RoutePlanner::Nano(route_planner)
    }
}

impl From<RotatingNanoIpRoutePlanner> for RoutePlanner {
    fn from(route_planner: RotatingNanoIpRoutePlanner) -> Self {
        RoutePlanner::RotatingNano(route_planner)
    }
}

impl From<BalancingIpRoutePlanner> for RoutePlanner {
    fn from(route_planner: BalancingIpRoutePlanner) -> Self {
        RoutePlanner::Balancing(route_planner)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kind of route planner.
pub enum RoutePlannerKind {
    /// IP address used is switched on ban. Recommended for IPv4 blocks or IPv6 blocks smaller than a /64.
    Rotating,
    /// IP address used is switched on clock update. Use with at least 1 /64 IPv6 block.
    Nano,
    /// IP address used is switched on clock update, rotates to a different /64 block on ban. Use with at least 2x /64 IPv6 blocks.
    RotatingNano,
    /// IP address used is selected at random per request. Recommended for larger IP blocks.
    Balancing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Route planner using a rotating IP address.
pub struct RotatingIpRoutePlanner {
    /// The ip block being used.
    pub ip_block: IPBlock,
    /// The failing addresses.
    pub failing_addresses: Vec<FailingAddress>,
    /// The number of rotations.
    pub rotate_index: String,
    /// The current offset in the block.
    pub ip_index: String,
    /// The current address being used.
    pub current_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Route planner using a nano IP address.
pub struct NanoIpRoutePlanner {
    /// The ip block being used.
    pub ip_block: IPBlock,
    /// The failing addresses.
    pub failing_addresses: Vec<FailingAddress>,
    /// The current offset in the ip block.
    pub current_address_index: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Route planner using a rotating nano IP address.
pub struct RotatingNanoIpRoutePlanner {
    /// The ip block being used.
    pub ip_block: IPBlock,
    /// The failing addresses.
    pub failing_addresses: Vec<FailingAddress>,
    /// The current offset in the ip block.
    pub current_address_index: String,
    /// The information in which /64 block ips are chosen. This number increases on each ban.
    pub block_index: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Route planner using a balancing IP address.
pub struct BalancingIpRoutePlanner {
    /// The ip block being used.
    pub ip_block: IPBlock,
    /// The failing addresses.
    pub failing_addresses: Vec<FailingAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "size")]
/// Represents an IP block with size and type.
pub enum IPBlock {
    #[serde(rename = "Inet4Address")]
    /// The ipv4 block type.
    Inet4(String),

    #[serde(rename = "Inet6Address")]
    /// The ipv6 block type.
    Inet6(String),
}

impl IPBlock {
    /// Get the inner value.
    pub fn content(&self) -> &str {
        match self {
            IPBlock::Inet4(content) => content,
            IPBlock::Inet6(content) => content,
        }
    }
}

impl From<IPBlock> for String {
    fn from(ip_block: IPBlock) -> Self {
        match ip_block {
            IPBlock::Inet4(content) => content,
            IPBlock::Inet6(content) => content,
        }
    }
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
