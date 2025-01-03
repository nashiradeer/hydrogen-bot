//! '/play' command registration and execution.

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
};
use tracing::{debug, error, info, warn};

use crate::{
    handler::{Response, ResponseType, ResponseValue},
    i18n::{
        serenity_command_description, serenity_command_name, serenity_command_option_description,
        serenity_command_option_name, t, t_vars,
    },
    music::PlayResult,
    PLAYER_MANAGER,
};

/// Executes the `/play` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Response<'a> {
    let guild_id = match interaction.guild_id {
        Some(v) => v,
        None => {
            info!(
                "(commands::play): the user {} is not in a guild",
                interaction.user.id
            );
            return Response::new(
                "play.embed_title",
                "error.not_in_guild",
                ResponseType::Error,
            );
        }
    };

    let manager = match PLAYER_MANAGER.get() {
        Some(v) => v,
        None => {
            error!("(commands::play): the manager is not initialized");
            return Response::new("play.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    let Some(query) = interaction
        .data
        .options
        .first()
        .and_then(|v| v.value.as_str())
    else {
        error!("(commands::play): cannot get the 'query' option");
        return Response::new("play.embed_title", "error.unknown", ResponseType::Error);
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        info!(
            "(commands::play): the user {} is not in a voice chat in the guild {}",
            interaction.user.id, guild_id
        );
        return Response::new(
            "play.embed_title",
            "error.unknown_voice_state",
            ResponseType::Error,
        );
    };

    let voice_manager = match songbird::get(context).await {
        Some(v) => v,
        None => {
            error!("(commands::play): cannot get the voice manager");
            return Response::new("play.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    let call = match voice_manager.get(guild_id) {
        Some(v) => {
            let has_connection = v.lock().await.current_connection().is_some();

            if !has_connection {
                // Join the voice channel.
                match voice_manager.join_gateway(guild_id, voice_channel_id).await {
                    Ok(v) => v.1,
                    Err(e) => {
                        warn!(
                            "(commands::play): cannot connect to the voice channel in the guild {}: {}",
                            guild_id, e
                        );
                        return Response::new(
                            "play.embed_title",
                            "error.cant_connect",
                            ResponseType::Error,
                        );
                    }
                }
            } else {
                v
            }
        }
        None => match voice_manager.join_gateway(guild_id, voice_channel_id).await {
            Ok(e) => e.1,
            Err(e) => {
                warn!(
                    "(commands::play): cannot connect to the voice channel in the guild {}: {}",
                    guild_id, e
                );
                return Response::new(
                    "play.embed_title",
                    "error.cant_connect",
                    ResponseType::Error,
                );
            }
        },
    };

    if let Some(connection_info) = call.lock().await.current_connection() {
        if let Some(channel_id) = connection_info.channel_id {
            if channel_id != voice_channel_id.into() {
                return Response::new(
                    "play.embed_title",
                    "error.not_in_voice_chat",
                    ResponseType::Error,
                );
            }
        }
    }

    let result = match manager
        .play(
            query,
            interaction.user.id,
            guild_id,
            interaction.channel_id,
            &interaction
                .guild_locale
                .clone()
                .unwrap_or(interaction.locale.clone()),
        )
        .await
    {
        Ok(e) => e,
        Err(e) => {
            warn!(
                "(commands::play): cannot play the music in the guild {}: {}",
                guild_id, e
            );
            return Response::new("play.embed_title", "error.unknown", ResponseType::Error);
        }
    };

    if result.count > 0 {
        Response::raw(
            ResponseValue::TranslationKey("play.embed_title"),
            ResponseValue::RawString(get_message(result, interaction)),
            ResponseType::Success,
        )
    } else if !result.truncated {
        Response::new("play.embed_title", "play.not_found", ResponseType::Error)
    } else {
        Response::new("play.embed_title", "play.truncated", ResponseType::Error)
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
            .dm_permission(false)
}

/// Get the message to send to the user.
fn get_message(result: PlayResult, interaction: &CommandInteraction) -> String {
    debug!("(commands::play): getting message for: {:?}", result);
    if let Some(track) = result.track {
        if result.playing && result.count == 1 {
            if let Some(url) = track.url {
                return t_vars(
                    &interaction.locale,
                    "play.play_single_url",
                    [
                        ("name", track.title),
                        ("author", track.author),
                        ("url", url),
                    ],
                );
            } else {
                return t_vars(
                    &interaction.locale,
                    "play.play_single",
                    [("name", track.title), ("author", track.author)],
                );
            }
        } else if result.count == 1 {
            if let Some(url) = track.url {
                return t_vars(
                    &interaction.locale,
                    "play.enqueue_single_url",
                    [
                        ("name", track.title),
                        ("author", track.author),
                        ("url", url),
                    ],
                );
            } else {
                return t_vars(
                    &interaction.locale,
                    "play.enqueue_single",
                    [("name", track.title), ("author", track.author)],
                );
            }
        } else if result.playing {
            if !result.truncated {
                if let Some(url) = track.url {
                    t_vars(
                        &interaction.locale,
                        "play.play_multi_url",
                        [
                            ("name", track.title),
                            ("author", track.author),
                            ("url", url),
                            ("count", result.count.to_string()),
                        ],
                    );
                } else {
                    return t_vars(
                        &interaction.locale,
                        "play.play_multi",
                        [
                            ("name", track.title),
                            ("author", track.author),
                            ("count", result.count.to_string()),
                        ],
                    );
                }
            } else if let Some(url) = track.url {
                return format!(
                    "{}\n\n{}",
                    t(&interaction.locale, "play.truncated_warn"),
                    t_vars(
                        &interaction.locale,
                        "play.play_multi_url",
                        [
                            ("name", track.title),
                            ("author", track.author),
                            ("url", url),
                            ("count", result.count.to_string()),
                        ]
                    ),
                );
            } else {
                return format!(
                    "{}\n\n{}",
                    t(&interaction.locale, "play.truncated_warn"),
                    t_vars(
                        &interaction.locale,
                        "play.play_multi",
                        [
                            ("name", track.title),
                            ("author", track.author),
                            ("count", result.count.to_string()),
                        ]
                    ),
                );
            }
        }
    }

    if result.truncated {
        return format!(
            "{}\n\n{}",
            t(&interaction.locale, "play.truncated_warn"),
            t_vars(
                &interaction.locale,
                "play.enqueue_multi",
                [("count", result.count.to_string())]
            ),
        );
    }

    t_vars(
        &interaction.locale,
        "play.enqueue_multi",
        [("count", result.count.to_string())],
    )
}
