//! Module for the Hydrogen's music player.

mod lavalink;
mod message;
mod player;

use hydrolink::{
    LoadResult, Rest, Track as LavalinkTrack, UpdatePlayer, UpdatePlayerTrack, VoiceState,
    cluster::Cluster,
};
use message::update_message;
pub use player::*;
use tokio::time::sleep;
use tracing::{Level, event};

use crate::utils::constants::{
    HYDROGEN_EMPTY_CHAT_TIMEOUT, HYDROGEN_QUEUE_LIMIT, HYDROGEN_SEARCH_PREFIXES,
};
use beef::lean::Cow;
use dashmap::DashMap;
use lavalink::{handle_lavalink, reconnect_node};
use rand::prelude::SliceRandom;
use serenity::all::{
    Cache, CacheHttp, ChannelId, ChannelType, GuildId, Http, UserId, VoiceServerUpdateEvent,
    VoiceState as SerenityVoiceState,
};
use songbird::{Songbird, error::JoinError};
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    result::Result as StdResult,
    sync::Arc,
    time::Duration,
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
    /// The bot user ID.
    user_id: UserId,
}

impl PlayerManager {
    /// Create a new player manager.
    pub async fn new(
        songbird: Arc<Songbird>,
        lavalink: Arc<Cluster>,
        cache: Arc<Cache>,
        http: Arc<Http>,
        user_id: UserId,
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
            user_id,
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
        player_template: PlayerTemplate,
    ) -> Result<()> {
        if !self.contains_player(guild_id) {
            self.create_player(guild_id, text_channel, locale, player_template)?;

            if let Some(player) = self.get_player_state(guild_id) {
                let (channel_id, message_id) =
                    update_message(self, guild_id, &player, false, false).await;

                self.players.alter(&guild_id, |_, p| Player {
                    channel_id,
                    message_id,
                    ..p
                });
            }
        }

        Ok(())
    }

    /// Create a player for the guild.
    fn create_player(
        &self,
        guild_id: GuildId,
        text_channel: ChannelId,
        locale: &str,
        template: PlayerTemplate,
    ) -> Result<()> {
        let node_id = self
            .lavalink
            .search_connected_node()
            .ok_or(Error::NoAvailableLavalink)?;

        self.players.insert(
            guild_id,
            template.into_player(node_id, locale, text_channel),
        );

        Ok(())
    }
    /// Check if the player exists for the guild.
    pub fn contains_player(&self, guild_id: GuildId) -> bool {
        self.players.contains_key(&guild_id)
    }

    /// Check if the connection exists for the guild.
    pub async fn contains_connection(&self, guild_id: GuildId) -> bool {
        if let Some(call) = self.songbird.get(guild_id) {
            let call_locked = call.lock().await;

            call_locked.current_connection().is_some() && call_locked.current_channel().is_some()
        } else {
            false
        }
    }

    /// Get the player connection for the guild.
    pub async fn get_connection(&self, guild_id: GuildId) -> Option<VoiceState> {
        let call = self.songbird.get(guild_id)?;

        let call_locked = call.lock().await;

        call_locked
            .current_connection()
            .map(|c| VoiceState::new(&c.token, &c.endpoint, &c.session_id))
    }

    /// Get the player state for the guild.
    pub fn get_player_state(&self, guild_id: GuildId) -> Option<PlayerState> {
        self.players.view(&guild_id, |_, p| p.into())
    }

    /// Get the current track playing in a player.
    pub fn get_current_track(&self, guild_id: GuildId) -> Option<Track> {
        self.players
            .view(&guild_id, |_, p| p.queue.get(p.current_track).cloned())
            .flatten()
    }

    /// Get the voice channel ID for the guild.
    ///
    /// This method will return `None` if the player does not exist too.
    pub async fn get_voice_channel_id(&self, guild_id: GuildId) -> Option<ChannelId> {
        let call = self.songbird.get(guild_id)?;

        let call_locked = call.lock().await;

        call_locked
            .current_channel()
            .map(|c| ChannelId::new(c.0.into()))
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

    /// Initialize the player for the guild, creating it if needed.
    async fn initialize_player(
        &self,
        guild_id: GuildId,
        text_channel: ChannelId,
        locale: &str,
        player_template: PlayerTemplate,
    ) -> Result<PlayerState> {
        let initializing = !self.contains_player(guild_id);

        if initializing {
            self.create_player(guild_id, text_channel, locale, player_template)?;
        }

        let player_state = self
            .get_player_state(guild_id)
            .ok_or(Error::InvalidGuildId)?;

        if initializing {
            let (channel_id, message_id) =
                update_message(self, guild_id, &player_state, false, true).await;

            self.players.alter(&guild_id, |_, p| Player {
                channel_id,
                message_id,
                ..p
            });
        }

        Ok(player_state)
    }

    /// Search for the music and fetch the result.
    async fn fetch(&self, query: &str, node_id: usize) -> Result<Option<FetchResult>> {
        let lavalink_node = &self.lavalink.nodes()[node_id];

        let songs = self.search(lavalink_node, query).await?;

        Ok(match songs {
            LoadResult::Search(tracks) => tracks.into_iter().nth(0).map(|t| FetchResult {
                selected: None,
                tracks: vec![t],
            }),
            LoadResult::Playlist(playlist) => Some(FetchResult {
                selected: if playlist.info.selected_track >= 0 {
                    Some(playlist.info.selected_track as usize)
                } else {
                    None
                },

                tracks: playlist.tracks.into_iter().collect(),
            }),
            LoadResult::Track(music) => Some(FetchResult {
                selected: None,
                tracks: vec![*music],
            }),
            LoadResult::Empty => None,
            LoadResult::Error(exception) => {
                event!(Level::WARN, error = ?exception, "failed to load track");

                None
            }
        })
    }

    /// Add the fetched tracks to the player's queue.
    fn add_queue(
        &self,
        guild_id: GuildId,
        fetch_result: FetchResult,
        requester: UserId,
        operation: AddQueueOperation,
    ) -> Result<AddQueueResult> {
        let mut player = self
            .players
            .get_mut(&guild_id)
            .ok_or(Error::PlayerNotFound)?;

        let old_queue_size = player.queue.len();

        let first_track_index = match operation {
            AddQueueOperation::End => player.queue.len(),
            AddQueueOperation::Next => player.current_track + 1,
        };

        let available_size = HYDROGEN_QUEUE_LIMIT - old_queue_size;

        if available_size == 0 {
            return Ok(AddQueueResult {
                count: 0,
                truncated: true,
                first_track_index: old_queue_size,
                selected: None,
            });
        }

        let old_tracks_size = fetch_result.tracks.len();

        let tracks = fetch_result
            .tracks
            .into_iter()
            .take(available_size)
            .map(|t| Track::from_track(t, requester))
            .collect::<Vec<_>>();

        let truncated = tracks.len() < old_tracks_size;

        let tracks_size = tracks.len();

        match operation {
            AddQueueOperation::End => player.queue.extend(tracks),
            AddQueueOperation::Next => {
                let current_track = player.current_track;

                let index = current_track + 1;

                player.queue.splice(index..index, tracks);
            }
        }

        let selected = fetch_result.selected.map(|i| {
            if let Some(new_index) = first_track_index.checked_add(i) {
                if new_index < player.queue.len() {
                    new_index
                } else {
                    first_track_index
                }
            } else {
                first_track_index
            }
        });

        Ok(AddQueueResult {
            count: tracks_size,
            truncated,
            first_track_index,
            selected,
        })
    }

    /// Check if the player is playing music.
    pub async fn is_playing(&self, guild_id: GuildId) -> Result<bool> {
        let player_state = self
            .get_player_state(guild_id)
            .ok_or(Error::PlayerNotFound)?;

        self.internal_is_playing(guild_id, &player_state).await
    }

    /// Internal method to check if the player is playing music.
    async fn internal_is_playing(
        &self,
        guild_id: GuildId,
        player_state: &PlayerState,
    ) -> Result<bool> {
        if player_state.track.is_none() {
            return Ok(false);
        }

        let player = self
            .lavalink
            .get_player(player_state.node_id, &guild_id.to_string())
            .await
            .map_err(Error::from)?;

        if let Some(player) = player {
            Ok(player.track.is_some())
        } else {
            Ok(false)
        }
    }

    /// Check if the player is playing music before updating and syncing it.
    async fn checked_update_sync(&self, guild_id: GuildId, track: usize) -> Result<SyncResult> {
        let player_state = self
            .get_player_state(guild_id)
            .ok_or(Error::PlayerNotFound)?;

        let is_playing = self.internal_is_playing(guild_id, &player_state).await?;

        let need_sync = matches!(player_state.loop_mode, LoopMode::All | LoopMode::None);

        let safe_track = self
            .players
            .view(&guild_id, |_, p| track < p.queue.len())
            .unwrap_or(false);

        if !is_playing && need_sync && safe_track {
            self.players.alter(&guild_id, |_, p| Player {
                current_track: track,
                ..p
            });

            let playing = self.sync(guild_id).await?;

            if playing {
                let current_track = self.get_current_track(guild_id);

                return Ok(SyncResult {
                    track: current_track,
                    playing,
                });
            }
        }

        Ok(SyncResult {
            track: self
                .players
                .view(&guild_id, |_, p| p.queue.get(track).cloned())
                .flatten(),
            playing: false,
        })
    }

    /// Update and sync the player forcefully.
    async fn forced_update_sync(&self, guild_id: GuildId, track: usize) -> Result<SyncResult> {
        let safe_track = self
            .players
            .view(&guild_id, |_, p| track < p.queue.len())
            .unwrap_or(false);

        if safe_track {
            self.players.alter(&guild_id, |_, p| Player {
                current_track: track,
                ..p
            });

            let playing = self.sync(guild_id).await?;

            if playing {
                let current_track = self.get_current_track(guild_id);

                return Ok(SyncResult {
                    track: current_track,
                    playing,
                });
            }
        }

        Ok(SyncResult {
            track: self
                .players
                .view(&guild_id, |_, p| p.queue.get(track).cloned())
                .flatten(),
            playing: false,
        })
    }

    /// Play a music or add it to the queue, initializing the player if needed.
    pub async fn play(&self, play_request: PlayRequest<'_>) -> Result<PlayResult> {
        let player_state = self
            .initialize_player(
                play_request.guild_id,
                play_request.text_channel,
                play_request.locale,
                play_request.player_template,
            )
            .await?;

        let Some(fetch_result) = self.fetch(play_request.music, player_state.node_id).await? else {
            return Ok(PlayResult {
                track: None,
                count: 0,
                playing: false,
                truncated: false,
            });
        };

        let add_queue_operation = match play_request.play_mode {
            PlayMode::AddToEnd => AddQueueOperation::End,
            _ => AddQueueOperation::Next,
        };

        let add_queue_result = self.add_queue(
            play_request.guild_id,
            fetch_result,
            play_request.requester,
            add_queue_operation,
        )?;

        let sync_result = if play_request.play_mode == PlayMode::PlayNow {
            self.forced_update_sync(
                play_request.guild_id,
                add_queue_result
                    .selected
                    .unwrap_or(add_queue_result.first_track_index),
            )
            .await
        } else {
            self.checked_update_sync(
                play_request.guild_id,
                add_queue_result
                    .selected
                    .unwrap_or(add_queue_result.first_track_index),
            )
            .await
        }?;

        Ok(PlayResult::merge(add_queue_result, sync_result))
    }

    /// Get the current playing time from the player.
    pub async fn time(&self, guild_id: GuildId) -> Result<Option<SeekResult>> {
        if !self.contains_player(guild_id) {
            return Err(Error::PlayerNotFound);
        }

        let node_id = self
            .players
            .view(&guild_id, |_, p| p.node_id)
            .ok_or(Error::PlayerNotFound)?;

        let player = self
            .lavalink
            .get_player(node_id, &guild_id.to_string())
            .await
            .map_err(Error::from)?;

        Ok(player.and_then(|p| {
            p.track.map(|t| SeekResult {
                position: t.info.position,
                total: t.info.length,
            })
        }))
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

        let position = time.as_millis() as u64;

        Ok(player.track.map(|t| SeekResult {
            position: if position > t.info.length {
                t.info.length
            } else {
                position
            },
            total: t.info.length,
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
        let is_playing = self.is_playing(guild_id).await?;

        if is_playing {
            let mut player_state = self
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

            player_state.paused = paused;

            let (channel_id, message_id) =
                update_message(self, guild_id, &player_state, true, false).await;

            self.players.alter(&guild_id, |_, p| Player {
                channel_id,
                message_id,
                paused,
                ..p
            });

            Ok(paused)
        } else {
            self.players
                .alter(&guild_id, |_, p| Player { paused: false, ..p });

            self.sync(guild_id).await?;

            Ok(false)
        }
    }

    /// Go to the previous track in the queue.
    pub async fn previous(&self, guild_id: GuildId) -> Result<Option<Track>> {
        let mut player = self
            .players
            .get_mut(&guild_id)
            .ok_or(Error::InvalidGuildId)?;

        player.current_track = if player.current_track > 0 {
            player.current_track - 1
        } else {
            player.queue.len() - 1
        };

        let current_track = player.queue.get(player.current_track).cloned();

        drop(player);

        self.sync(guild_id).await?;

        Ok(current_track)
    }

    /// Go to the next track in the queue.
    pub async fn skip(&self, guild_id: GuildId) -> Result<Option<Track>> {
        let mut player = self
            .players
            .get_mut(&guild_id)
            .ok_or(Error::InvalidGuildId)?;

        player.current_track = (player.current_track + 1) % player.queue.len();

        let current_track = player.queue.get(player.current_track).cloned();

        drop(player);

        self.sync(guild_id).await?;

        Ok(current_track)
    }

    /// Starts the player, requesting the Lavalink node to play the music.
    async fn sync(&self, guild_id: GuildId) -> Result<bool> {
        let player_state = self
            .players
            .view(&guild_id, |_, p| {
                p.queue
                    .get(p.current_track)
                    .map(|t| (t.track.clone(), p.paused, p.node_id))
            })
            .flatten();

        if let Some((song, paused, node_id)) = player_state {
            let voice = self.get_connection(guild_id).await;

            let update_player = UpdatePlayer {
                voice,
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

        let is_me = voice_state.user_id == self.cache.current_user().id;

        if is_me {
            if voice_state.channel_id.is_some() {
                if let Some(ref player_state) = player_state {
                    let voice = self
                        .get_connection(guild_id)
                        .await
                        .map(|c| VoiceState::new(&c.token, &c.endpoint, &voice_state.session_id));

                    self.update_connection(voice, player_state.node_id, guild_id)
                        .await?;
                }
            } else {
                self.destroy(guild_id).await?;
                return Ok(true);
            }
        }

        let voice_channel_id = if is_me {
            voice_state.channel_id
        } else {
            self.get_voice_channel_id(guild_id).await
        };

        if let Some(channel_id) = voice_channel_id {
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
                    let is_playing = self
                        .internal_is_playing(guild_id, &player_state)
                        .await
                        .unwrap_or(true);

                    let (channel_id, message_id) =
                        update_message(self, guild_id, &player_state, is_playing, thinking).await;

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

        if self.contains_player(guild_id) {
            let player_state = self.get_player_state(guild_id);

            if let Some(player_state) = player_state {
                let voice = self.get_connection(guild_id).await.and_then(|c| {
                    voice_server
                        .endpoint
                        .map(|e| VoiceState::new(&voice_server.token, &e, &c.session_id))
                });

                self.update_connection(voice, player_state.node_id, guild_id)
                    .await?;
            }
        }

        Ok(true)
    }

    async fn update_connection(
        &self,
        voice: Option<VoiceState>,
        node_id: usize,
        guild_id: GuildId,
    ) -> Result<()> {
        if voice.is_some() {
            let update_player = UpdatePlayer {
                voice,
                ..Default::default()
            };

            self.lavalink
                .update_player(node_id, &guild_id.to_string(), &update_player, true)
                .await
                .map_err(Error::from)?;
        }

        Ok(())
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

    /// Check if the player exists, is playing the last track, and is with
    fn should_autoplay(&self, guild_id: GuildId) -> bool {
        let Some(player) = self.players.get(&guild_id) else {
            return false;
        };

        if player.current_track + 1 >= player.queue.len() {
            matches!(player.loop_mode, LoopMode::Autoplay)
        } else {
            false
        }
    }

    /// Uses the player's loop mode to determine the next track to play.
    pub async fn next_track(&self, guild_id: GuildId) -> Result<()> {
        if self.should_autoplay(guild_id) && self.autoplay(guild_id).await? {
            return Ok(());
        }

        let Some(mut player) = self.players.get_mut(&guild_id) else {
            return Ok(());
        };

        let (new_index, should_pause, need_sync) = match player.loop_mode {
            LoopMode::Single => (player.current_track, false, true),
            LoopMode::All => ((player.current_track + 1) % player.queue.len(), false, true),
            LoopMode::AutoPause => {
                if player.current_track + 1 >= player.queue.len() {
                    (player.queue.len() - 1, true, false)
                } else {
                    (player.current_track + 1, true, false)
                }
            }
            _ => {
                if player.current_track + 1 >= player.queue.len() {
                    (player.queue.len() - 1, false, false)
                } else {
                    (player.current_track + 1, false, true)
                }
            }
        };

        player.current_track = new_index;
        player.paused = should_pause;

        drop(player);

        if need_sync {
            self.sync(guild_id).await?;
        } else {
            self.update_message(guild_id).await;
        }

        Ok(())
    }

    /// Autoplay the next track using YouTube Mix, returning `true` if it was successful.
    async fn autoplay(&self, guild_id: GuildId) -> Result<bool> {
        if self
            .players
            .view(&guild_id, |_, p| p.queue.len() >= HYDROGEN_QUEUE_LIMIT)
            .unwrap_or(true)
        {
            return Ok(false);
        }

        event!(Level::DEBUG, guild_id = ?guild_id, "autoplay has been triggered");

        let Some(current_track) = self.players.view(&guild_id, |_, p| p.current_track) else {
            return Ok(false);
        };

        let Some(track) = self
            .get_track_from_youtube_mix(guild_id, current_track)
            .await?
        else {
            return Ok(false);
        };

        let fetch_result = FetchResult {
            selected: Some(0),
            tracks: vec![track],
        };

        let add_queue_result =
            self.add_queue(guild_id, fetch_result, self.user_id, AddQueueOperation::End)?;

        self.forced_update_sync(
            guild_id,
            add_queue_result
                .selected
                .unwrap_or(add_queue_result.first_track_index),
        )
        .await?;

        Ok(true)
    }

    /// Update the player message.
    pub async fn update_message(&self, guild_id: GuildId) {
        let player_state = self.get_player_state(guild_id);

        if let Some(player_state) = player_state {
            let is_playing = self
                .internal_is_playing(guild_id, &player_state)
                .await
                .unwrap_or(true);

            let (channel_id, message_id) =
                update_message(self, guild_id, &player_state, is_playing, false).await;

            self.players.alter(&guild_id, |_, p| Player {
                channel_id,
                message_id,
                ..p
            });
        }
    }

    /// Shuffle the player's queue.
    pub fn shuffle(&self, guild_id: GuildId) -> Result<()> {
        let mut player = self
            .players
            .get_mut(&guild_id)
            .ok_or(Error::PlayerNotFound)?;

        let old_index = player.current_track;

        let current_track = player.queue.swap_remove(old_index);

        player.queue.shuffle(&mut rand::rng());

        player.queue.insert(0, current_track);

        player.current_track = 0;

        Ok(())
    }

    /// Convert the load result to the identifier of the first track.
    fn get_identifier(&self, load_result: LoadResult) -> Option<String> {
        match load_result {
            LoadResult::Track(track) => Some(track.info.identifier.clone()),
            LoadResult::Playlist(playlist) => {
                playlist.tracks.first().map(|t| t.info.identifier.clone())
            }
            LoadResult::Search(tracks) => tracks.first().map(|t| t.info.identifier.clone()),
            _ => None,
        }
    }

    /// Get the YouTube ID from a query.
    async fn get_youtube_id_from_query(&self, query: &str) -> Result<Option<String>> {
        let node_id = self
            .lavalink
            .search_connected_node()
            .ok_or(Error::NoAvailableLavalink)?;

        let node = &self.lavalink.nodes()[node_id];

        let result = node
            .load_track(&format!("ytsearch:{}", query))
            .await
            .map_err(Error::from)?;

        Ok(self.get_identifier(result))
    }

    /// Get the YouTube ID from an ISRC code.
    async fn get_youtube_id_from_isrc(&self, isrc: &str) -> Result<Option<String>> {
        let node_id = self
            .lavalink
            .search_connected_node()
            .ok_or(Error::NoAvailableLavalink)?;

        let node = &self.lavalink.nodes()[node_id];

        let result = node
            .load_track(&format!("ytsearch:\"{}\"", isrc))
            .await
            .map_err(Error::from)?;

        Ok(self.get_identifier(result))
    }

    /// Convert a track from queue to YouTube ID.
    async fn get_youtube_id(&self, guild_id: GuildId, index: usize) -> Result<Option<String>> {
        let youtube_id = self
            .players
            .view(&guild_id, |_, p| {
                p.queue.get(index).map(|t| t.youtube_id.clone())
            })
            .ok_or(Error::PlayerNotFound)?
            .flatten();

        if youtube_id.is_none() {
            if let Some(isrc) = self
                .players
                .view(&guild_id, |_, p| p.queue.get(index).map(|t| t.isrc.clone()))
                .ok_or(Error::PlayerNotFound)?
                .flatten()
            {
                return self
                    .get_youtube_id_from_isrc(&isrc)
                    .await
                    .inspect(|youtube_id| self.update_youtube_id(guild_id, index, youtube_id));
            }

            if let Some(track) = self
                .players
                .view(&guild_id, |_, p| {
                    p.queue.get(index).map(|t| t.track.clone())
                })
                .ok_or(Error::PlayerNotFound)?
            {
                return self
                    .get_youtube_id_from_query(&track)
                    .await
                    .inspect(|youtube_id| self.update_youtube_id(guild_id, index, youtube_id));
            }
        }

        Ok(youtube_id)
    }

    /// Update the YouTube ID for a track in the player's queue.
    fn update_youtube_id(&self, guild_id: GuildId, index: usize, youtube_id: &Option<String>) {
        if let Some(youtube_id) = youtube_id {
            self.players.alter(&guild_id, |_, mut p| {
                if let Some(track) = p.queue.get_mut(index) {
                    if track.youtube_id.is_none() {
                        track.youtube_id = Some(youtube_id.clone());
                    }
                }
                p
            });
        }
    }

    /// Get the YouTube ID from a YouTube mix.
    async fn get_track_from_youtube_mix(
        &self,
        guild_id: GuildId,
        index: usize,
    ) -> Result<Option<LavalinkTrack>> {
        let Some(youtube_id) = self.get_youtube_id(guild_id, index).await? else {
            return Ok(None);
        };

        event!(Level::DEBUG, guild_id = ?guild_id, youtube_id = youtube_id, "getting track from youtube mix");

        let node_id = self
            .lavalink
            .search_connected_node()
            .ok_or(Error::NoAvailableLavalink)?;

        let node = &self.lavalink.nodes()[node_id];

        let result = node
            .load_track(&format!(
                "https://www.youtube.com/watch?v={0}&list=RD{0}",
                youtube_id
            ))
            .await
            .map_err(Error::from)?;

        let tracks = match result {
            LoadResult::Playlist(playlist) => playlist.tracks.into_iter(),
            LoadResult::Search(tracks) => tracks.into_iter(),
            _ => return Ok(None),
        };

        let track = self
            .get_non_duplicated_track(guild_id, tracks.skip(1))
            .await?;

        event!(Level::DEBUG, guild_id = ?guild_id, track = ?track, "got track from youtube mix");

        Ok(track)
    }

    /// Get the YouTube ID from a Lavalink track.
    async fn get_youtube_id_from_lavalink_track<'a>(
        &self,
        track: &'a LavalinkTrack,
    ) -> Option<Cow<'a, str>> {
        if track
            .info
            .source_name
            .as_ref()
            .is_some_and(|s| s == "youtube")
        {
            return Some(Cow::borrowed(&track.info.identifier));
        } else {
            if let Some(isrc) = track.info.isrc.as_ref() {
                if let Ok(Some(youtube_id)) = self.get_youtube_id_from_isrc(isrc).await.inspect_err(
                    |e| event!(Level::WARN, error = ?e, "failed to get youtube id from isrc"),
                ) {
                    return Some(Cow::owned(youtube_id));
                }
            }

            if let Ok(Some(youtube_id)) = self
                .get_youtube_id_from_query(&track.info.title)
                .await
                .inspect_err(
                    |e| event!(Level::WARN, error = ?e, "failed to get youtube id from query"),
                )
            {
                return Some(Cow::owned(youtube_id));
            }
        };

        None
    }

    /// Get a non-duplicated track from the provided tracks.
    async fn get_non_duplicated_track<T: Iterator<Item = LavalinkTrack>>(
        &self,
        guild_id: GuildId,
        tracks: T,
    ) -> Result<Option<LavalinkTrack>> {
        for track in tracks {
            event!(Level::TRACE, guild_id = ?guild_id, track = ?track, "checking track for duplication");

            let is_duplicated = match self.get_youtube_id_from_lavalink_track(&track).await {
                Some(id) => self.contains_track_by_youtube_id(guild_id, &id).await,
                None => false,
            };

            if !is_duplicated {
                return Ok(Some(track));
            }
        }

        Ok(None)
    }

    /// Check if the player contains a track by its YouTube ID.
    async fn contains_track_by_youtube_id(&self, guild_id: GuildId, youtube_id: &str) -> bool {
        let track_count = self
            .players
            .view(&guild_id, |_, p| p.queue.len())
            .unwrap_or(0);

        for i in 0..track_count {
            let track_youtube_id = self
                .get_youtube_id(guild_id, i)
                .await
                .inspect_err(|e| event!(Level::WARN, error = ?e, "failed to get youtube id"))
                .unwrap_or(None);

            if let Some(id) = track_youtube_id {
                if id == youtube_id {
                    return true;
                }
            }
        }

        false
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
    Lavalink(hydrolink::Error),
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

impl From<hydrolink::Error> for Error {
    fn from(e: hydrolink::Error) -> Self {
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
