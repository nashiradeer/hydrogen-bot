//! '/play' command registration and execution.

use beef::lean::Cow;
use serenity::all::{
    ChannelId, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    GuildId,
};
use songbird::{Call, Songbird};
use std::default::Default;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{event, Level};

use crate::i18n::t_all;
use crate::music::{PlayMode, PlayRequest};
use crate::{
    i18n::{
        serenity_command_description, serenity_command_name, serenity_command_option_description,
        serenity_command_option_name, t, t_vars,
    },
    music::PlayResult,
    utils, PLAYER_MANAGER,
};

/// Executes the `/play` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    let Some(guild_id) = interaction.guild_id else {
        event!(Level::WARN, "interaction.guild_id is None");
        return Cow::borrowed(t(&interaction.locale, "error.not_in_guild"));
    };

    let Some(manager) = PLAYER_MANAGER.get() else {
        event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    };

    let Some(query) = interaction
        .data
        .options
        .first()
        .and_then(|v| v.value.as_str())
    else {
        event!(Level::WARN, "no query provided");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    };

    let mode_option = interaction
        .data
        .options
        .get(1)
        .and_then(|v| v.value.as_str());

    let mode = match mode_option {
        Some("add_next") => PlayMode::AddToNext,
        Some("play_now") => PlayMode::PlayNow,
        _ => PlayMode::AddToEnd,
    };

    let (voice_manager, voice_channel_id) = match utils::get_voice_essentials(
        context,
        &interaction.locale,
        guild_id,
        interaction.user.id,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => return e,
    };

    let call = match voice_manager.get(guild_id) {
        Some(v) => {
            let has_connection = v.lock().await.current_connection().is_some();

            if !has_connection {
                match join_gateway(
                    &voice_manager,
                    guild_id,
                    voice_channel_id,
                    &interaction.locale,
                )
                .await
                {
                    Ok(v) => v,
                    Err(e) => return e,
                }
            } else {
                v
            }
        }
        None => match join_gateway(
            &voice_manager,
            guild_id,
            voice_channel_id,
            &interaction.locale,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => return e,
        },
    };

    if let Some(connection_info) = call.lock().await.current_connection() {
        if let Some(channel_id) = connection_info.channel_id {
            if channel_id != voice_channel_id.into() {
                return Cow::borrowed(t(&interaction.locale, "error.not_in_voice_channel"));
            }
        }
    }

    let play_request = PlayRequest {
        music: query,
        requester: interaction.user.id,
        guild_id,
        text_channel: interaction.channel_id,
        locale: &interaction.locale,
        player_template: Default::default(),
        play_mode: mode,
    };

    let result = match manager.play(play_request).await {
        Ok(e) => e,
        Err(e) => {
            event!(Level::ERROR, error = ?e, guild_id = %guild_id, "cannot play the track");
            return Cow::borrowed(t(&interaction.locale, "error.unknown"));
        }
    };

    if result.count > 0 {
        generate_message(result, interaction)
    } else if !result.truncated {
        Cow::borrowed(t(&interaction.locale, "play.not_found"))
    } else {
        Cow::borrowed(t(&interaction.locale, "play.truncated"))
    }
}

/// Creates the `/join` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("play");

    command = serenity_command_name("play.name", command);
    command = serenity_command_description("play.description", command);

    command
        .description(
            "Request music to be played, enqueuing it in the queue or playing immediately if empty.",
        )
        .add_option({
            let mut option = CreateCommandOption::new(
                CommandOptionType::String,
                "query",
                "A music or playlist URL, or a search term.",
            )
                .required(true);

            option =
                serenity_command_option_name("play.query_name", option);
            option = serenity_command_option_description(
                "play.query_description",
                option,
            );

            option
        })
        .add_option({
            let mut option = CreateCommandOption::new(
                CommandOptionType::String,
                "mode",
                "The mode to play the song in.",
            )
                .required(false)
                .add_string_choice_localized("Add To End", "add_to_end", t_all("play.mode_end"))
                .add_string_choice_localized("Add Next", "add_next", t_all("play.mode_next"))
                .add_string_choice_localized("Play Now", "play_now", t_all("play.mode_now"));

            option =
                serenity_command_option_name("play.mode_name", option);
            option = serenity_command_option_description(
                "play.mode_description",
                option,
            );

            option
        })
        .dm_permission(false)
}

/// Joins the voice channel.
async fn join_gateway<'a>(
    voice_manager: &Arc<Songbird>,
    guild_id: GuildId,
    voice_channel_id: ChannelId,
    locale: &str,
) -> Result<Arc<Mutex<Call>>, Cow<'a, str>> {
    voice_manager.join_gateway(guild_id, voice_channel_id).await.map(|e| e.1).map_err(|e| {
        event!(Level::INFO, voice_channel_id = %voice_channel_id, error = ?e, "cannot join the voice channel");
        Cow::borrowed(t(locale, "error.cant_connect"))
    })
}

/// Generates the message from the result from the player.
fn generate_message<'a>(result: PlayResult, interaction: &CommandInteraction) -> Cow<'a, str> {
    event!(
        Level::TRACE,
        result = ?result,
        "generating message from PlayResult"
    );
    if let Some(track) = result.track {
        if result.playing && result.count == 1 {
            return if let Some(url) = track.url {
                t_vars(
                    &interaction.locale,
                    "play.play_single_url",
                    [track.title, track.author, url],
                )
            } else {
                t_vars(
                    &interaction.locale,
                    "play.play_single",
                    [track.title, track.author],
                )
            };
        } else if result.count == 1 {
            return if let Some(url) = track.url {
                t_vars(
                    &interaction.locale,
                    "play.enqueue_single_url",
                    [track.title, track.author, url],
                )
            } else {
                t_vars(
                    &interaction.locale,
                    "play.enqueue_single",
                    [track.title, track.author],
                )
            };
        } else if result.playing {
            return if !result.truncated {
                if let Some(url) = track.url {
                    t_vars(
                        &interaction.locale,
                        "play.play_multi_url",
                        [track.title, track.author, result.count.to_string(), url],
                    )
                } else {
                    t_vars(
                        &interaction.locale,
                        "play.play_multi",
                        [track.title, track.author, result.count.to_string()],
                    )
                }
            } else if let Some(url) = track.url {
                Cow::owned(format!(
                    "{}\n\n{}",
                    t(&interaction.locale, "play.truncated_warn"),
                    t_vars(
                        &interaction.locale,
                        "play.play_multi_url",
                        [track.title, track.author, result.count.to_string(), url]
                    ),
                ))
            } else {
                Cow::owned(format!(
                    "{}\n\n{}",
                    t(&interaction.locale, "play.truncated_warn"),
                    t_vars(
                        &interaction.locale,
                        "play.play_multi",
                        [track.title, track.author, result.count.to_string()]
                    ),
                ))
            };
        }
    }

    if result.truncated {
        return Cow::owned(format!(
            "{}\n\n{}",
            t(&interaction.locale, "play.truncated_warn"),
            t_vars(
                &interaction.locale,
                "play.enqueue_multi",
                [result.count.to_string()]
            ),
        ));
    }

    t_vars(
        &interaction.locale,
        "play.enqueue_multi",
        [result.count.to_string()],
    )
}
