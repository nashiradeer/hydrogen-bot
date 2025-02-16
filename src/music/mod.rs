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
    /// The connections to be used by the players.
    connections: Arc<DashMap<GuildId, PlayerConnection>>,
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
        let connections = Arc::new(DashMap::<GuildId, PlayerConnection>::new());

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
            connections,
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
        if !self.contains_player(guild_id) {
            self.inner_init(guild_id, text_channel, locale).await?;

            if let Some(player) = self.get_player_state(guild_id) {
                let (channel_id, message_id) = update_message(self, guild_id, &player, false).await;
                self.players.alter(&guild_id, |_, p| Player {
                    channel_id,
                    message_id,
                    ..p
                });
            }
        }

        Ok(())
    }

    /// Internal [Self::init] logic to be shared between methods.
    async fn inner_init(
        &self,
        guild_id: GuildId,
        text_channel: ChannelId,
        locale: &str,
    ) -> Result<()> {
        let node_id = self
            .lavalink
            .search_connected_node()
            .ok_or(Error::NoAvailableLavalink)?;

        self.players
            .insert(guild_id, Player::new_normal(node_id, locale, text_channel));

        Ok(())
    }

    /// Check if the player exists for the guild.
    pub fn contains_player(&self, guild_id: GuildId) -> bool {
        self.players.contains_key(&guild_id)
    }

    /// Check if the connection exists for the guild.
    pub fn contains_connection(&self, guild_id: GuildId) -> bool {
        self.connections.contains_key(&guild_id)
    }

    /// Check if the connection is ready for the guild.
    pub fn connection_ready(&self, guild_id: GuildId) -> bool {
        self.connections
            .get(&guild_id)
            .is_some_and(|c| c.is_ready())
    }

    /// Get the player state for the guild.
    pub fn get_player_state(&self, guild_id: GuildId) -> Option<PlayerState> {
        self.players.view(&guild_id, |_, p| p.into())
    }

    /// Get the current track playing in a player.
    pub fn get_current_track(&self, guild_id: GuildId) -> Option<Track> {
        self.players
            .view(&guild_id, |_, p| {
                p.primary_queue.get(p.currrent_track).cloned()
            })
            .flatten()
    }

    /// Get the voice channel ID for the guild.
    ///
    /// This method will return `None` if the player does not exist too.
    pub fn get_voice_channel_id(&self, guild_id: GuildId) -> Option<ChannelId> {
        if !self.contains_player(guild_id) {
            return None;
        }

        self.connections
            .view(&guild_id, |_, c| c.serenity_channel_id())
            .flatten()
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
        let initializing = !self.contains_player(guild_id);

        if initializing {
            self.inner_init(guild_id, text_channel, locale).await?;
        }

        let player_state = self
            .get_player_state(guild_id)
            .ok_or(Error::InvalidGuildId)?;

        if initializing {
            let (channel_id, message_id) =
                update_message(self, guild_id, &player_state, true).await;
            self.players.alter(&guild_id, |_, p| Player {
                channel_id,
                message_id,
                ..p
            });
        }

        let lavalink_node = &self.lavalink.nodes()[player_state.node_id];

        let songs = self.search(lavalink_node, music).await?;

        match songs {
            LoadResult::Search(tracks) => {
                if let Some(music) = tracks.into_iter().nth(0) {
                    self.inner_play(guild_id, requester, None, vec![music])
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
                    Some(playlist.info.selected_track),
                    playlist.tracks,
                )
                .await
            }
            LoadResult::Track(music) => {
                self.inner_play(guild_id, requester, None, vec![*music])
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
        selected_track: Option<i32>,
        raw_tracks: Vec<LavalinkTrack>,
    ) -> Result<PlayResult> {
        let raw_tracks_size = raw_tracks.len();

        let original_queue_size = self
            .players
            .view(&guild_id, |_, p| p.primary_queue.len())
            .unwrap_or(0);

        let available_size = HYDROGEN_QUEUE_LIMIT - original_queue_size;

        let tracks = raw_tracks
            .into_iter()
            .take(available_size)
            .map(|t| {
                let mut track = Track::from(t);
                track.requester = requester;

                track
            })
            .collect::<Vec<_>>();

        let tracks_size = tracks.len();

        event!(
            Level::DEBUG,
            original_queue_size = original_queue_size,
            available_size = available_size,
            raw_tracks_size = raw_tracks_size,
            tracks_size = tracks_size,
            "inserting tracks into the player"
        );

        let truncated = tracks_size < raw_tracks_size;

        let mut player = self
            .players
            .get_mut(&guild_id)
            .ok_or(Error::InvalidGuildId)?;

        player.primary_queue.extend(tracks);

        let player_state = PlayerState::from(player.value());

        drop(player);

        let mut playing = false;

        let lavalink_not_playing = match self
            .lavalink
            .get_player(player_state.node_id, &guild_id.to_string())
            .await
        {
            Ok(v) => v.map_or(true, |p| p.track.is_none()),
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

        let mut this_play_track = self
            .players
            .view(&guild_id, |_, p| {
                p.primary_queue.get(original_queue_size).cloned()
            })
            .flatten();

        if lavalink_not_playing {
            let mut index =
                match original_queue_size.overflowing_add(selected_track.unwrap_or(0) as usize) {
                    (v, false) => v,
                    (_, true) => {
                        event!(
                            Level::WARN,
                            starting_index = original_queue_size,
                            selected_track = selected_track,
                            "index overflowed"
                        );
                        original_queue_size
                    }
                };

            let mut player = self
                .players
                .get_mut(&guild_id)
                .ok_or(Error::InvalidGuildId)?;

            if index >= player.primary_queue.len() {
                index = original_queue_size;
            }

            player.currrent_track = index;
            player.paused = false;

            drop(player);

            playing = self.start_player(guild_id).await?;

            if playing {
                this_play_track = self.get_current_track(guild_id);
            }
        }

        Ok(PlayResult {
            track: this_play_track,
            count: tracks_size,
            playing,
            truncated,
        })
    }

    /// Seek the player to a certain time.
    pub async fn seek(&self, guild_id: GuildId, time: Duration) -> Result<Option<SeekResult>> {
        if !self.contains_player(guild_id) {
            return Err(Error::PlayerNotFound);
        }

        let update_player = UpdatePlayer::default().set_position(time.as_millis() as u64);

        let node_id = self
            .players
            .view(&guild_id, |_, p| p.node_id)
            .ok_or(Error::PlayerNotFound)?;

        let player = self
            .lavalink
            .update_player(node_id, &guild_id.to_string(), &update_player, true)
            .await
            .map_err(Error::from)?;

        Ok(player.track.map(|t| SeekResult {
            position: t.info.position,
            total: t.info.length,
            track: Track::from(t),
        }))
    }

    /// Get the loop mode for the guild.
    pub fn get_loop_mode(&self, guild_id: GuildId) -> Option<LoopMode> {
        self.players.view(&guild_id, |_, p| p.loop_mode)
    }

    /// Set the loop mode for the guild.
    pub async fn set_loop_mode(&self, guild_id: GuildId, loop_mode: LoopMode) {
        self.players
            .alter(&guild_id, |_, p| Player { loop_mode, ..p });

        self.update_message(guild_id).await;
    }

    /// Get the pause state for the guild.
    pub fn get_pause(&self, guild_id: GuildId) -> Option<bool> {
        self.players.view(&guild_id, |_, p| p.paused)
    }

    /// Set the pause state for the guild.
    pub async fn set_pause(&self, guild_id: GuildId, paused: bool) -> Result<bool> {
        let player_state = self
            .get_player_state(guild_id)
            .ok_or(Error::PlayerNotFound)?;

        let update_player = UpdatePlayer::default().set_paused(paused);

        self.lavalink
            .update_player(
                player_state.node_id,
                &guild_id.to_string(),
                &update_player,
                true,
            )
            .await
            .map_err(Error::from)?;

        let (channel_id, message_id) = update_message(self, guild_id, &player_state, false).await;

        self.players.alter(&guild_id, |_, p| Player {
            channel_id,
            message_id,
            paused,
            ..p
        });

        Ok(true)
    }

    /// Go to the previous track in the queue.
    pub async fn previous(&self, guild_id: GuildId) -> Result<Option<Track>> {
        let mut player = self
            .players
            .get_mut(&guild_id)
            .ok_or(Error::InvalidGuildId)?;

        player.currrent_track = if player.currrent_track > 0 {
            player.currrent_track - 1
        } else {
            player.primary_queue.len() - 1
        };

        let current_track = player.primary_queue.get(player.currrent_track).cloned();

        drop(player);

        self.start_player(guild_id).await?;

        Ok(current_track)
    }

    /// Go to the next track in the queue.
    pub async fn skip(&self, guild_id: GuildId) -> Result<Option<Track>> {
        let mut player = self
            .players
            .get_mut(&guild_id)
            .ok_or(Error::InvalidGuildId)?;

        player.currrent_track = (player.currrent_track + 1) % player.primary_queue.len();

        let current_track = player.primary_queue.get(player.currrent_track).cloned();

        drop(player);

        self.start_player(guild_id).await?;

        Ok(current_track)
    }

    /// Starts the player, requesting the Lavalink node to play the music.
    async fn start_player(&self, guild_id: GuildId) -> Result<bool> {
        let player_state = self
            .players
            .view(&guild_id, |_, p| {
                p.primary_queue
                    .get(p.currrent_track)
                    .map(|t| (t.track.clone(), p.paused, p.node_id))
            })
            .flatten();

        if let Some((song, paused, node_id)) = player_state {
            let voice_state = self
                .connections
                .view(&guild_id, |_, c| {
                    TryInto::<VoiceState>::try_into(c.clone()).ok()
                })
                .flatten();

            let update_player = UpdatePlayer {
                voice: voice_state,
                ..Default::default()
            }
            .set_track(UpdatePlayerTrack::default().set_encoded(&song))
            .set_paused(paused);

            self.lavalink
                .update_player(node_id, &guild_id.to_string(), &update_player, false)
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
        _: Option<&SerenityVoiceState>,
        voice_state: &SerenityVoiceState,
    ) -> Result<bool> {
        let guild_id = voice_state.guild_id.ok_or(Error::InvalidGuildId)?;

        let player_state = self.get_player_state(guild_id);

        if voice_state.user_id == self.cache.current_user().id {
            if let Some(channel_id) = voice_state.channel_id {
                if !self.contains_connection(guild_id) {
                    self.connections.insert(
                        guild_id,
                        PlayerConnection::default()
                            .set_session_id(&voice_state.session_id)
                            .set_channel_id(channel_id.into()),
                    );
                } else {
                    self.connections.alter(&guild_id, |_k, v| {
                        v.set_session_id(&voice_state.session_id)
                            .set_channel_id(channel_id.into())
                    });
                }

                if let Some(ref player_state) = player_state {
                    let voice = self
                        .connections
                        .view(&guild_id, |_, c| c.clone().try_into().ok())
                        .flatten();

                    let update_player = UpdatePlayer {
                        voice,
                        ..Default::default()
                    };

                    self.lavalink
                        .update_player(
                            player_state.node_id,
                            &guild_id.to_string(),
                            &update_player,
                            true,
                        )
                        .await?;
                }
            } else {
                self.destroy(guild_id).await?;
                return Ok(true);
            }
        }

        let channel_id = self
            .connections
            .view(&guild_id, |_, v| v.serenity_channel_id())
            .flatten();

        if let Some(channel_id) = channel_id {
            let member_count = {
                let cache_ref = self
                    .cache
                    .guild(guild_id)
                    .ok_or(Error::GuildChannelNotFound)?;

                let channel = cache_ref
                    .channels
                    .get(&channel_id)
                    .ok_or(Error::GuildChannelNotFound)?;

                if channel.kind == ChannelType::Voice || channel.kind == ChannelType::Stage {
                    let members_len = channel
                        .members(self.cache.as_ref())
                        .map_err(Error::from)?
                        .len();

                    Some(members_len)
                } else {
                    None
                }
            };

            if let Some(members_count) = member_count {
                let thinking = if members_count <= 1 {
                    self.timed_destroy(guild_id, Duration::from_secs(HYDROGEN_EMPTY_CHAT_TIMEOUT))
                        .await;

                    true
                } else {
                    self.cancel_destroy(guild_id);

                    false
                };

                let new_player_state = self.get_player_state(guild_id);

                if let Some(player_state) = new_player_state {
                    let (channel_id, message_id) =
                        update_message(self, guild_id, &player_state, thinking).await;

                    self.players.alter(&guild_id, |_, p| Player {
                        channel_id,
                        message_id,
                        ..p
                    });
                }
            }
        }

        Ok(true)
    }

    /// Handles the voice server update event, updating the player's connection.
    pub async fn update_voice_server(&self, voice_server: VoiceServerUpdateEvent) -> Result<bool> {
        let guild_id = voice_server.guild_id.ok_or(Error::InvalidGuildId)?;

        if !self.contains_connection(guild_id) {
            let mut player_connection = PlayerConnection::default().set_token(&voice_server.token);

            player_connection.endpoint = voice_server.endpoint;

            self.connections.insert(guild_id, player_connection);
        } else {
            self.connections.alter(&guild_id, |_k, v| PlayerConnection {
                token: Some(voice_server.token.clone()),
                endpoint: voice_server.endpoint,
                ..v
            });
        }

        if self.contains_player(guild_id) {
            let player_state = self.get_player_state(guild_id);

            if let Some(player_state) = player_state {
                let voice = self
                    .connections
                    .view(&guild_id, |_, c| c.clone().try_into().ok())
                    .flatten();

                let update_player = UpdatePlayer {
                    voice,
                    ..Default::default()
                };

                self.lavalink
                    .update_player(
                        player_state.node_id,
                        &guild_id.to_string(),
                        &update_player,
                        true,
                    )
                    .await?;
            }
        }

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

        if let Some((message_id, text_channel)) = player.message_id.zip(player.channel_id) {
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
        self.players.alter(&guild_id, |_, mut player| {
            if player.destroy_handle.is_none() {
                let self_clone = self.clone();

                player.destroy_handle = Some(tokio::spawn(async move {
                    sleep(duration).await;
                    _ = self_clone.destroy(guild_id).await;
                }));
            }

            player
        });
    }

    /// Cancel the destroy task for the player.
    fn cancel_destroy(&self, guild_id: GuildId) {
        self.players.alter(&guild_id, |_, mut player| {
            if let Some(handle) = player.destroy_handle.take() {
                handle.abort();
            }

            player
        });
    }

    /// Uses the player's loop mode to determine the next track to play.
    pub async fn next_track(&self, guild_id: GuildId) -> Result<()> {
        let Some(mut player) = self.players.get_mut(&guild_id) else {
            return Ok(());
        };

        let (new_index, should_pause) = match player.loop_mode {
            LoopMode::None => {
                if player.currrent_track + 1 >= player.primary_queue.len() {
                    (player.primary_queue.len() - 1, true)
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
                    (player.primary_queue.len() - 1, true)
                } else {
                    (player.currrent_track + 1, true)
                }
            }
        };

        player.currrent_track = new_index;
        player.paused = should_pause;

        drop(player);

        self.start_player(guild_id).await?;

        Ok(())
    }

    /// Update the player message.
    pub async fn update_message(&self, guild_id: GuildId) {
        let player_state = self.get_player_state(guild_id);
        if let Some(player_state) = player_state {
            let (channel_id, message_id) =
                update_message(self, guild_id, &player_state, false).await;

            self.players.alter(&guild_id, |_, p| Player {
                channel_id,
                message_id,
                ..p
            });
        }
    }
}

impl CacheHttp for PlayerManager {
    fn http(&self) -> &Http {
        &self.http
    }

    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }
}

/// Result type for the player manager.
pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
/// Errors that can occur when using the player manager.
pub enum Error {
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
    /// There's no player for the guild.
    PlayerNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lavalink(e) => write!(f, "Lavalink error: {}", e),
            Self::Join(e) => write!(f, "Join error: {}", e),
            Self::Serenity(e) => write!(f, "Serenity error: {}", e),
            Self::NoAvailableLavalink => write!(f, "There's no available Lavalink node"),
            Self::InvalidGuildId => write!(f, "Invalid guild ID"),
            Self::GuildChannelNotFound => write!(f, "Guild channel was not found"),
            Self::PlayerNotFound => write!(f, "Player not found"),
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
