use serenity::all::{
    ButtonStyle, ChannelId, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedAuthor,
    CreateMessage, EditMessage, GuildId, MessageId, ReactionType,
};
use tracing::{event, Level};

use crate::{
    i18n::{t, t_vars},
    utils::constants::{HYDROGEN_EMPTY_CHAT_TIMEOUT, HYDROGEN_PRIMARY_COLOR},
};

use super::{PlayerManager, PlayerState, Track};

/// Whether to disable the previous button.
const DISABLE_PREVIOUS: bool = false;
/// Whether to disable the play/pause button.
const DISABLE_PAUSE: bool = false;
/// Whether to disable the skip button.
const DISABLE_SKIP: bool = false;
/// Whether to disable the stop button.
const DISABLE_STOP: bool = false;
/// Whether to disable the loop button.
const DISABLE_LOOP: bool = false;
/// Whether to disable the shuffle button.
const DISABLE_SHUFFLE: bool = true;
/// Whether to disable the queue button.
const DISABLE_QUEUE: bool = true;

/// Updates the player message.
pub async fn update_message(
    manager: &PlayerManager,
    guild_id: GuildId,
    player: &PlayerState,
    thinking: bool,
) -> (Option<ChannelId>, Option<MessageId>) {
    event!(
        Level::TRACE,
        thinking = thinking,
        player_state = ?player,
        "updating player message"
    );

    let track = player.track.as_ref();

    let state = PlayerMessageState::detect_state(track, thinking);

    let title = generate_title(player, track);
    let description = generate_message(player, track);
    let url = generate_url(player, track);
    let author = generate_author(manager, player, guild_id).await;
    let components = generate_components(player, &state);

    let embed = generate_embed(
        &description,
        title,
        url,
        track.and_then(|track| track.thumbnail.clone()),
        author,
    );

    if let Some(channel_id) = player.text_channel {
        if let Some(message_id) = player.message_id {
            match channel_id
                .edit_message(
                    &manager,
                    message_id,
                    EditMessage::new()
                        .embed(embed.clone())
                        .components(components.clone()),
                )
                .await
            {
                Ok(msg) => {
                    event!(
                        Level::DEBUG,
                        player = ?player,
                        guild_id = ?guild_id,
                        "player message updated"
                    );
                    return (Some(channel_id), Some(msg.id));
                }
                Err(e) => {
                    event!(
                        Level::INFO,
                        error = %e,
                        player = ?player,
                        guild_id = ?guild_id,
                        "cannot edit player message, sending a new one"
                    );
                }
            }
        }

        return match channel_id
            .send_message(
                &manager,
                CreateMessage::new().add_embed(embed).components(components),
            )
            .await
        {
            Ok(message) => {
                event!(
                    Level::DEBUG,
                    player = ?player,
                    guild_id = ?guild_id,
                    "player message sent"
                );
                (Some(channel_id), Some(message.id))
            }
            Err(e) => {
                event!(
                    Level::INFO,
                    error = %e,
                    player = ?player,
                    guild_id = ?guild_id,
                    "cannot send player message"
                );
                (None, None)
            }
        };
    }

    (player.text_channel, player.message_id)
}

/// Generates the embed for the player message.
fn generate_embed(
    description: &str,
    title: Option<String>,
    url: Option<String>,
    thumbnail: Option<String>,
    author: Option<CreateEmbedAuthor>,
) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .description(description)
        .color(HYDROGEN_PRIMARY_COLOR);

    if let Some(title) = title {
        embed = embed.title(title);
    }

    if let Some(author) = author {
        embed = embed.author(author);
    }

    if let Some(url) = url {
        embed = embed.url(url);
    }

    if let Some(thumbnail) = thumbnail {
        embed = embed.thumbnail(thumbnail)
    }

    embed
}

/// Generates the title for the embed.
fn generate_title(player: &PlayerState, track: Option<&Track>) -> Option<String> {
    if !player.has_destroy_handle {
        track.map(|track| format!("**{}**", track.title))
    } else {
        None
    }
}

/// Generates the message description.
fn generate_message(player: &PlayerState, track: Option<&Track>) -> String {
    if player.has_destroy_handle {
        t_vars(
            &player.locale,
            "player.timeout",
            [("time", HYDROGEN_EMPTY_CHAT_TIMEOUT.to_string())],
        )
    } else {
        match track {
            Some(track) => track.author.clone(),
            None => t(&player.locale, "player.empty").to_owned(),
        }
    }
}

/// Generates the URL for the embed.
fn generate_url(player: &PlayerState, track: Option<&Track>) -> Option<String> {
    if player.has_destroy_handle {
        track.and_then(|track| track.url.clone())
    } else {
        None
    }
}

/// Generates the author for the embed.
async fn generate_author(
    manager: &PlayerManager,
    player: &PlayerState,
    guild_id: GuildId,
) -> Option<CreateEmbedAuthor> {
    if player.has_destroy_handle {
        return None;
    }

    let valid_track = player.track.as_ref()?;

    let user = valid_track.requester.to_user(manager).await.ok();

    let user_name = match guild_id
        .member(manager, valid_track.requester)
        .await
        .ok()
        .and_then(|v| v.nick)
    {
        Some(name) => name,
        None => {
            let user = valid_track.requester.to_user(manager).await.ok()?;
            user.global_name.unwrap_or(user.name.clone())
        }
    };

    let mut author = CreateEmbedAuthor::new(user_name);

    if let Some(avatar) = user.and_then(|v| v.avatar_url()) {
        author = author.icon_url(avatar);
    }

    Some(author)
}

/// Generates the components for the player message.
fn generate_components(player: &PlayerState, state: &PlayerMessageState) -> Vec<CreateActionRow> {
    let main_row_style = match state.is_thinking() {
        true => ButtonStyle::Secondary,
        false => ButtonStyle::Primary,
    };

    let pause_icon = match player.paused {
        true => '▶',
        false => '⏸',
    };

    Vec::from(&[
        CreateActionRow::Buttons(Vec::from(&[
            CreateButton::new("prev")
                .disabled(DISABLE_PREVIOUS || !state.is_playing())
                .emoji('⏮')
                .style(main_row_style),
            CreateButton::new("pause")
                .disabled(DISABLE_PAUSE || !state.is_playing())
                .emoji(pause_icon)
                .style(main_row_style),
            CreateButton::new("skip")
                .disabled(DISABLE_SKIP || !state.is_playing())
                .emoji('⏭')
                .style(main_row_style),
        ])),
        CreateActionRow::Buttons(Vec::from(&[
            CreateButton::new("loop")
                .disabled(DISABLE_LOOP || state.is_thinking())
                .emoji(ReactionType::Unicode(player.loop_mode.next().to_string()))
                .style(ButtonStyle::Secondary),
            CreateButton::new("stop")
                .disabled(DISABLE_STOP || state.is_thinking())
                .emoji('⏹')
                .style(ButtonStyle::Danger),
            CreateButton::new("queue")
                .disabled(DISABLE_QUEUE || !state.is_playing())
                .emoji(ReactionType::Unicode("ℹ️".to_owned()))
                .style(ButtonStyle::Secondary),
            CreateButton::new("shuffle")
                .disabled(DISABLE_SHUFFLE || !state.is_playing())
                .emoji('🔀')
                .style(ButtonStyle::Secondary),
        ])),
    ])
}

/// Represents the state of the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerMessageState {
    /// The player is empty.
    Empty,
    /// The player is playing.
    Playing,
    /// The player is thinking.
    Thinking,
}

impl PlayerMessageState {
    /// Detects the state of the player.
    pub fn detect_state(track: Option<&Track>, thinking: bool) -> Self {
        if thinking {
            Self::Thinking
        } else if track.is_some() {
            Self::Playing
        } else {
            Self::Empty
        }
    }

    /// Returns whether the player is empty.
    pub fn _is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns whether the player is playing.
    pub fn is_playing(&self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Returns whether the player is thinking.
    pub fn is_thinking(&self) -> bool {
        matches!(self, Self::Thinking)
    }
}
