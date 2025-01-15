//! Module for the Hydrogen's music player.

mod lavalink;
mod message;
mod player;

use message::update_message;
pub use player::*;
use tokio::time::sleep;
use tracing::{event, Level};

use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    result::Result as StdResult,
    sync::Arc,
    time::Duration,
};

use dashmap::DashMap;
use lavalink::{handle_lavalink, reconnect_node};
use serenity::all::{
    Cache, CacheHttp, ChannelId, ChannelType, GuildId, Http, UserId, VoiceServerUpdateEvent,
    VoiceState as SerenityVoiceState,
};
use songbird::{error::JoinError, Songbird};

use crate::{
    lavalink::{
        cluster::Cluster, Error as LavalinkError, LoadResult, Rest, Track as LavalinkTrack,
        UpdatePlayer, UpdatePlayerTrack, VoiceState,
    },
    utils::constants::{
        HYDROGEN_EMPTY_CHAT_TIMEOUT, HYDROGEN_QUEUE_LIMIT, HYDROGEN_SEARCH_PREFIXES,
    },
};

#[derive(Debug, Clone)]
/// The player manager.
pub struct PlayerManager {
    /// The players.
    players: Arc<DashMap<GuildId, Player>>,
    /// The voice manager.
    ///
    /// This [Arc] comes from outside the player manager.
    songbird: Arc<Songbird>,
    /// The Lavalink cluster.
    lavalink: Arc<Cluster>,
    /// Serenity's cache.
    ///
    /// This [Arc] comes from outside the player manager.
    cache: Arc<Cache>,
    /// Serenity's HTTP client.
    ///
    /// This [Arc] comes from outside the player manager.
    http: Arc<Http>,
}

impl PlayerManager {
    /// Create a new player manager.
    pub async fn new(
        songbird: Arc<Songbird>,
        lavalink: Arc<Cluster>,
        cache: Arc<Cache>,
        http: Arc<Http>,
    ) -> Self {
        let players = Arc::new(DashMap::<GuildId, Player>::new());

        for i in 0..lavalink.nodes().len() {
            event!(Level::DEBUG, node_id = i, "connecting to Lavalink...");
            if let Err(e) = lavalink.connect(i).await {
                event!(Level::ERROR, node_id = i, error = ?e, "failed to connect to Lavalink");
                reconnect_node(lavalink.clone(), i);
            }
            event!(Level::INFO, node_id = i, "connected to Lavalink");
        }

        let me = Self {
            players,
            songbird,
            lavalink,
            cache,
            http,
        };

        handle_lavalink(me.clone());

        me
    }

    /// Initialize a new player for the guild.
    pub async fn init(
        &self,
        guild_id: GuildId,
        text_channel: ChannelId,
        locale: &str,
    ) -> Result<()> {
        self.inner_init(guild_id, text_channel, locale).await?;

        let mut player = self.players.get_mut(&guild_id).unwrap();
        update_message(self, &mut player, guild_id, false).await;

        Ok(())
    }

    /// Internal [Self::init] logic to be shared between methods.
    async fn inner_init(
        &self,
        guild_id: GuildId,
        text_channel: ChannelId,
        locale: &str,
    ) -> Result<()> {
        let call = self
            .songbird
            .get(guild_id)
            .ok_or(Error::VoiceNotConnected)?;

        let call_lock = call.lock().await;

        let connection_info = call_lock
            .current_connection()
            .ok_or(Error::VoiceNotConnected)?;

        let lavalink_node_id = self
            .lavalink
            .search_connected_node()
            .await
            .ok_or(Error::NoAvailableLavalink)?;

        self.players.insert(
            guild_id,
            Player {
                currrent_track: 0,
                destroy_handle: None,
                endpoint: connection_info.endpoint.clone(),
                locale: locale.to_owned(),
                loop_mode: Default::default(),
                node_id: lavalink_node_id,
                message_id: None,
                paused: false,
                primary_queue: Vec::new(),
                _secondary_queue: None,
                text_channel: Some(text_channel),
                voice_channel: ChannelId::new(
                    connection_info
                        .channel_id
                        .ok_or(Error::VoiceNotConnected)?
                        .0
                        .into(),
                ),
                session_id: connection_info.session_id.clone(),
                token: connection_info.token.clone(),
            },
        );

        Ok(())
    }

    /// Check if the player exists for the guild.
    pub fn contains_player(&self, guild_id: GuildId) -> bool {
        self.players.contains_key(&guild_id)
    }

    /// Search for the music using multiple prefixes.
    pub async fn search(&self, node: &Rest, music: &str) -> Result<LoadResult> {
        let result = node.load_track(music).await.map_err(Error::from)?;

        if result.is_empty() {
            for prefix in HYDROGEN_SEARCH_PREFIXES.iter() {
                let result = node
                    .load_track(&format!("{}{}", prefix, music))
                    .await
                    .map_err(Error::from)?;

                if !result.is_empty() {
                    return Ok(result);
                }
            }
        }

        Ok(result)
    }

    /// Play a music or add it to the queue, initializing the player if needed.
    pub async fn play(
        &self,
        music: &str,
        requester: UserId,
        guild_id: GuildId,
        text_channel: ChannelId,
        locale: &str,
    ) -> Result<PlayResult> {
        if !self.contains_player(guild_id) {
            self.inner_init(guild_id, text_channel, locale).await?;
        }

        let mut player = self.players.get_mut(&guild_id).unwrap();
        update_message(self, &mut player, guild_id, true).await;

        let lavalink_node = &self.lavalink.nodes()[player.node_id];

        let songs = self.search(lavalink_node, music).await?;

        match songs {
            LoadResult::Search(tracks) => {
                if let Some(music) = tracks.into_iter().nth(0) {
                    self.inner_play(guild_id, requester, None, &mut player, vec![music])
                        .await
                } else {
                    Ok(PlayResult {
                        track: None,
                        count: 0,
                        playing: false,
                        truncated: false,
                    })
                }
            }
            LoadResult::Playlist(playlist) => {
                self.inner_play(
                    guild_id,
                    requester,
                    Some(playlist.info.selected_track as usize),
                    &mut player,
                    playlist.tracks,
                )
                .await
            }
            LoadResult::Track(music) => {
                self.inner_play(guild_id, requester, None, &mut player, vec![music])
                    .await
            }
            LoadResult::Empty => Ok(PlayResult {
                track: None,
                count: 0,
                playing: false,
                truncated: false,
            }),
            LoadResult::Error(exception) => {
                event!(Level::WARN, error = ?exception, "failed to load track");

                Ok(PlayResult {
                    track: None,
                    count: 0,
                    playing: false,
                    truncated: false,
                })
            }
        }
    }

    /// Internal [Self::play] logic to be shared between methods.
    async fn inner_play(
        &self,
        guild_id: GuildId,
        requester: UserId,
        selected_track: Option<usize>,
        player: &mut Player,
        tracks: Vec<LavalinkTrack>,
    ) -> Result<PlayResult> {
        let mut truncated = false;

        let starting_index = player.primary_queue.len();

        for music in tracks {
            if player.primary_queue.len() < HYDROGEN_QUEUE_LIMIT {
                let mut track = Track::from(music);
                track.requester = requester;

                player.primary_queue.push(track);
            } else {
                truncated = true;
                break;
            }
        }

        let mut playing = false;

        let lavalink_not_playing = match self
            .lavalink
            .get_player(player.node_id, &guild_id.to_string())
            .await
        {
            Ok(v) => v.track.is_none(),
            Err(e) => {
                if let LavalinkError::Lavalink(ref er) = e {
                    if er.status != 404 {
                        return Err(e.into());
                    }
                } else {
                    return Err(e.into());
                }

                true
            }
        };

        let mut this_play_track = player.primary_queue.get(starting_index).cloned();

        if lavalink_not_playing {
            let mut index = match starting_index.overflowing_add(selected_track.unwrap_or(0)) {
                (v, false) => v,
                (_, true) => {
                    event!(
                        Level::WARN,
                        starting_index = starting_index,
                        selected_track = selected_track,
                        "index overflowed"
                    );
                    starting_index
                }
            };

            if index >= player.primary_queue.len() {
                index = starting_index;
            }

            player.currrent_track = index;
            player.paused = false;

            playing = self.start_player(guild_id, &player).await?;
            if playing {
                this_play_track = player.primary_queue.get(index).cloned();
            }
        }

        Ok(PlayResult {
            track: this_play_track,
            count: player.primary_queue.len() - starting_index,
            playing,
            truncated,
        })
    }

    /// Starts the player, requesting the Lavalink node to play the music.
    async fn start_player(&self, guild_id: GuildId, player: &Player) -> Result<bool> {
        if let Some(music) = player.primary_queue.get(player.currrent_track) {
            let update_player = UpdatePlayer {
                track: Some(UpdatePlayerTrack {
                    encoded: Some(Some(music.track.clone())),
                    ..Default::default()
                }),
                voice: Some(VoiceState {
                    endpoint: player.endpoint.clone(),
                    session_id: player.session_id.clone(),
                    token: player.token.clone(),
                }),
                paused: Some(player.paused),
                ..Default::default()
            };

            self.lavalink
                .update_player(player.node_id, &guild_id.to_string(), &update_player, None)
                .await
                .map_err(Error::from)?;

            event!(
                Level::DEBUG,
                guild_id = ?guild_id,
                "player started"
            );

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Handles the voice state update event, updating the player's connection.
    pub async fn update_voice_state(
        &self,
        old_voice_state: Option<&SerenityVoiceState>,
        voice_state: &SerenityVoiceState,
    ) -> Result<bool> {
        let guild_id = voice_state.guild_id.ok_or(Error::InvalidGuildId)?;
        let Some(mut player) = self.players.get_mut(&guild_id) else {
            return Ok(false);
        };

        if old_voice_state.is_some() && voice_state.user_id == self.cache.current_user().id {
            if let Some(channel_id) = voice_state.channel_id {
                player.voice_channel = channel_id;
                player.session_id = voice_state.session_id.clone();
            } else {
                drop(player);
                self.destroy(guild_id).await?;
                return Ok(true);
            }
        }

        let channel = self
            .cache
            .guild(guild_id)
            .ok_or(Error::GuildChannelNotFound)?
            .channels
            .get(&ChannelId::new(player.voice_channel.into()))
            .ok_or(Error::GuildChannelNotFound)?
            .clone();

        if channel.kind == ChannelType::Voice || channel.kind == ChannelType::Stage {
            let members_count = channel
                .members(self.cache.clone())
                .map_err(Error::from)?
                .len();

            if members_count <= 1 {
                self.inner_timed_destroy(
                    guild_id,
                    &mut player,
                    Duration::from_secs(HYDROGEN_EMPTY_CHAT_TIMEOUT),
                );

                update_message(self, &mut player, guild_id, true).await;
            } else {
                self.cancel_destroy(&mut player).await;
                update_message(self, &mut player, guild_id, false).await;
            }
        }

        Ok(true)
    }

    /// Handles the voice server update event, updating the player's connection.
    pub async fn update_voice_server(&self, voice_server: VoiceServerUpdateEvent) -> Result<bool> {
        let guild_id = voice_server.guild_id.ok_or(Error::InvalidGuildId)?;

        let Some(mut player) = self.players.get_mut(&guild_id) else {
            return Ok(false);
        };

        player.token = voice_server.token;

        if let Some(endpoint) = voice_server.endpoint {
            player.endpoint = endpoint;
        }

        let update_player = UpdatePlayer {
            voice: Some(VoiceState {
                endpoint: player.endpoint.clone(),
                session_id: player.session_id.clone(),
                token: player.token.clone(),
            }),
            ..Default::default()
        };

        self.lavalink
            .update_player(player.node_id, &guild_id.to_string(), &update_player, None)
            .await
            .map_err(Error::from)?;

        Ok(true)
    }

    /// Destroy the player, stopping the music and leaving the voice channel.
    pub async fn destroy(&self, guild_id: GuildId) -> Result<()> {
        let Some((_, player)) = self.players.remove(&guild_id) else {
            return Ok(());
        };

        self.songbird.leave(guild_id).await.map_err(Error::from)?;

        self.lavalink
            .destroy_player(player.node_id, &guild_id.to_string())
            .await
            .map_err(Error::from)?;

        if let Some((message_id, text_channel)) = player.message_id.zip(player.text_channel) {
            self.http
                .delete_message(
                    text_channel,
                    message_id,
                    Some("Message auto-deleted by timeout."),
                )
                .await
                .map_err(Error::from)?;
        }

        if let Some(destroy_handle) = player.destroy_handle {
            destroy_handle.abort();
        }

        Ok(())
    }

    /// Destroy the player after a certain duration.
    pub async fn timed_destroy(&self, guild_id: GuildId, duration: Duration) {
        if let Some(mut player) = self.players.get_mut(&guild_id) {
            self.inner_timed_destroy(guild_id, &mut player, duration);
        }
    }

    /// Internal [Self::timed_destroy] logic to be shared between methods.
    fn inner_timed_destroy(&self, guild_id: GuildId, player: &mut Player, duration: Duration) {
        if player.destroy_handle.is_none() {
            let self_clone = self.clone();

            player.destroy_handle = Some(tokio::spawn(async move {
                sleep(duration).await;
                _ = self_clone.destroy(guild_id).await;
            }));
        }
    }

    /// Cancel the destroy task for the player.
    async fn cancel_destroy(&self, player: &mut Player) {
        if let Some(handle) = player.destroy_handle.take() {
            handle.abort();
        }
    }

    /// Uses the player's loop mode to determine the next track to play.
    pub async fn next_track(&self, guild_id: GuildId) -> Result<()> {
        let Some(mut player) = self.players.get_mut(&guild_id) else {
            return Ok(());
        };

        let (new_index, should_pause) = match player.loop_mode {
            LoopMode::None => {
                if player.currrent_track + 1 >= player.primary_queue.len() {
                    (0, true)
                } else {
                    (player.currrent_track + 1, false)
                }
            }
            LoopMode::Single => (player.currrent_track, false),
            LoopMode::All => (
                player.currrent_track + 1 % player.primary_queue.len(),
                false,
            ),
            LoopMode::Autopause => {
                if player.currrent_track + 1 >= player.primary_queue.len() {
                    (0, true)
                } else {
                    (player.currrent_track + 1, false)
                }
            }
        };

        player.currrent_track = new_index;
        player.paused = should_pause;

        self.start_player(guild_id, &player).await?;

        update_message(self, &mut player, guild_id, false).await;

        Ok(())
    }

    /// Update the player message.
    pub async fn update_message(&self, guild_id: GuildId) {
        if let Some(mut player) = self.players.get_mut(&guild_id) {
            update_message(self, &mut player, guild_id, false).await;
        }
    }
}

impl CacheHttp for PlayerManager {
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }

    fn http(&self) -> &Http {
        &self.http
    }
}

/// Result type for the player manager.
pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
/// Errors that can occur when using the player manager.
pub enum Error {
    /// There's no voice connection for the guild in the voice manager.
    VoiceNotConnected,
    /// There's no available Lavalink node.
    NoAvailableLavalink,
    /// Error from the Lavalink node.
    Lavalink(crate::lavalink::Error),
    /// Invalid guild ID.
    InvalidGuildId,
    /// Error when joining a voice channel.
    Join(JoinError),
    /// Error from the Serenity library.
    Serenity(serenity::Error),
    /// The guild channel was not found.
    GuildChannelNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lavalink(e) => write!(f, "Lavalink error: {}", e),
            Self::Join(e) => write!(f, "Join error: {}", e),
            Self::Serenity(e) => write!(f, "Serenity error: {}", e),
            Self::VoiceNotConnected => write!(
                f,
                "There's no voice connection for the guild in the voice manager"
            ),
            Self::NoAvailableLavalink => write!(f, "There's no available Lavalink node"),
            Self::InvalidGuildId => write!(f, "Invalid guild ID"),
            Self::GuildChannelNotFound => write!(f, "Guild channel was not found"),
        }
    }
}

impl From<crate::lavalink::Error> for Error {
    fn from(e: crate::lavalink::Error) -> Self {
        Self::Lavalink(e)
    }
}

impl From<JoinError> for Error {
    fn from(e: JoinError) -> Self {
        Self::Join(e)
    }
}

impl From<serenity::Error> for Error {
    fn from(e: serenity::Error) -> Self {
        Self::Serenity(e)
    }
}

impl StdError for Error {}
