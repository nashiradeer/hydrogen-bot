//! '/loop' command registration and execution.

use beef::lean::Cow;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
};
use tracing::{Level, event};

use crate::i18n::{
    serenity_command_description, serenity_command_name, serenity_command_option_description,
    serenity_command_option_name, t_all,
};
use crate::shared::SharedInteraction;
use crate::utils::delete_player_message;
use crate::{PLAYER_MANAGER, i18n::t, music::LoopMode, utils};

/// Executes the `/loop` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    let Some(guild_id) = interaction.guild_id else {
        event!(Level::WARN, "interaction.guild_id is None");
        return Cow::borrowed(t(&interaction.locale, "error.not_in_guild"));
    };

    let Some(manager) = PLAYER_MANAGER.get() else {
        event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    };

    let loop_option = interaction
        .data
        .options
        .first()
        .and_then(|v| v.value.as_str());

    let loop_selected = match loop_option {
        Some("single") => LoopMode::Single,
        Some("all") => LoopMode::All,
        Some("auto_pause") => LoopMode::AutoPause,
        Some("autoplay") => LoopMode::Autoplay,
        _ => LoopMode::None,
    };

    let voice_channel_id =
        match utils::get_voice_channel(context, &interaction.locale, guild_id, interaction.user.id)
        {
            Ok(v) => v,
            Err(e) => return e,
        };

    let player_state = manager.get_voice_channel_id(guild_id).await;

    if let Some(my_channel_id) = player_state {
        if my_channel_id == voice_channel_id {
            manager.set_loop_mode(guild_id, loop_selected).await;

            let loop_type_translation_key = match loop_selected {
                LoopMode::None => "loop.normal",
                LoopMode::AutoPause => "loop.pause",
                LoopMode::Single => "loop.music",
                LoopMode::All => "loop.queue",
                LoopMode::Autoplay => "loop.autoplay",
            };

            Cow::borrowed(t(&interaction.locale, loop_type_translation_key))
        } else {
            Cow::borrowed(t(&interaction.locale, "error.not_in_voice_channel"))
        }
    } else {
        delete_player_message(context, &SharedInteraction::from(interaction)).await;

        Cow::borrowed(t(&interaction.locale, "error.player_not_exists"))
    }
}

/// Creates the `/loop` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("loop");

    command = serenity_command_name("loop.name", command);
    command = serenity_command_description("loop.description", command);

    command
        .description("Changes the loop mode of the player.")
        .add_option({
            let mut option = CreateCommandOption::new(
                CommandOptionType::String,
                "mode",
                "The loop mode to set.",
            )
            .required(true)
            .add_string_choice_localized("Default", "default", t_all("loop.mode_default"))
            .add_string_choice_localized("Single", "single", t_all("loop.mode_single"))
            .add_string_choice_localized("All", "all", t_all("loop.mode_all"))
            .add_string_choice_localized("Auto Pause", "auto_pause", t_all("loop.mode_auto_pause"))
            .add_string_choice_localized(
                "Autoplay",
                "autoplay",
                t_all("loop.mode_autoplay"),
            );

            option = serenity_command_option_name("loop.mode_name", option);
            option = serenity_command_option_description("loop.mode_description", option);

            option
        })
        .dm_permission(false)
}
