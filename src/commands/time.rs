//! '/time' command registration and execution.

use beef::lean::Cow;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
};
use tracing::{event, Level};

use crate::i18n::t;
use crate::{
    i18n::{
        serenity_command_description, serenity_command_name, serenity_command_option_description,
        serenity_command_option_name, t_vars,
    },
    utils,
    utils::{
        progress_bar,
        time_parsers::{semicolon_syntax, suffix_syntax},
        time_to_string,
    },
    PLAYER_MANAGER,
};

/// Executes the `/time` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    let Some(guild_id) = interaction.guild_id else {
        event!(Level::WARN, "interaction.guild_id is None");
        return Cow::borrowed(t(&interaction.locale, "error.not_in_guild"));
    };

    let Some(manager) = PLAYER_MANAGER.get() else {
        event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    };

    let time_option = interaction
        .data
        .options
        .first()
        .and_then(|v| v.value.as_str());

    let voice_channel_id =
        match utils::get_voice_channel(context, &interaction.locale, guild_id, interaction.user.id)
        {
            Ok(v) => v,
            Err(e) => return e,
        };

    let my_channel_id = manager.get_voice_channel_id(guild_id).await;

    if let Some(my_channel_id) = my_channel_id {
        if my_channel_id == voice_channel_id {
            let possible_seek = if let Some(time) = time_option {
                let seek_time = match suffix_syntax(time) {
                    Some(v) => v,
                    None => match semicolon_syntax(time) {
                        Some(v) => v,
                        None => {
                            event!(Level::INFO, syntax = time, "invalid syntax provided");
                            return Cow::borrowed(t(&interaction.locale, "error.invalid_syntax"));
                        }
                    },
                };

                manager.seek(guild_id, seek_time).await
            } else {
                manager.time(guild_id).await
            };

            let seek_result = match possible_seek {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Cow::borrowed(t(&interaction.locale, "error.empty_queue"));
                }
                Err(e) => {
                    event!(Level::ERROR, error = ?e, "cannot seek the player");
                    return Cow::borrowed(t(&interaction.locale, "error.unknown"));
                }
            };

            let current_time = time_to_string(seek_result.position / 1000);
            let total_time = time_to_string(seek_result.total / 1000);
            let progress_bar = progress_bar(seek_result.position, seek_result.total);

            t_vars(
                &interaction.locale,
                "time.result",
                [current_time, total_time, progress_bar],
            )
        } else {
            Cow::borrowed(t(&interaction.locale, "error.not_in_voice_channel"))
        }
    } else {
        Cow::borrowed(t(&interaction.locale, "error.player_not_exists"))
    }
}

/// Creates the `/time` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("time");

    command = serenity_command_name("time.name", command);
    command = serenity_command_description("time.description", command);

    command
        .description("See or change the current time of the playing track.")
        .add_option({
            let mut option = CreateCommandOption::new(
                CommandOptionType::String,
                "time",
                "Time in seconds or a supported syntax.",
            )
            .required(false);

            option = serenity_command_option_name("time.time_name", option);
            option = serenity_command_option_description("time.time_description", option);

            option
        })
        .dm_permission(false)
}
