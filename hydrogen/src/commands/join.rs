//! '/join' command registration and execution.

use beef::lean::Cow;
use serenity::all::{CommandOptionType, CreateCommandOption};
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};
use tracing::{Level, event};

use crate::i18n::{serenity_command_option_description, serenity_command_option_name, t, t_all};
use crate::music::PlayerTemplate;
use crate::{
    LOADED_COMMANDS, PLAYER_MANAGER,
    i18n::{serenity_command_description, serenity_command_name, t_vars},
    utils,
};

/// Executes the `/join` command.
pub async fn execute<'a>(context: &Context, interaction: &CommandInteraction) -> Cow<'a, str> {
    let Some(guild_id) = interaction.guild_id else {
        event!(Level::WARN, "interaction.guild_id is None");
        return Cow::borrowed(t(&interaction.locale, "error.not_in_guild"));
    };

    let Some(manager) = PLAYER_MANAGER.get() else {
        event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    };

    if manager.contains_player(guild_id) {
        event!(Level::INFO, "player already exists");
        return Cow::borrowed(t(&interaction.locale, "error.player_exists"));
    }

    let template_option = interaction
        .data
        .options
        .first()
        .and_then(|v| v.value.as_str());

    let template = match template_option {
        Some("music") => PlayerTemplate::Music,
        Some("queue") => PlayerTemplate::Queue,
        Some("manual") => PlayerTemplate::Manual,
        Some("rpg") => PlayerTemplate::Rpg,
        Some("autoplay") => PlayerTemplate::Autoplay,
        _ => PlayerTemplate::Default,
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

    if let Err(e) = voice_manager.join_gateway(guild_id, voice_channel_id).await {
        event!(Level::INFO, voice_channel_id = %voice_channel_id, error = %e, "cannot join the voice channel");
        return Cow::borrowed(t(&interaction.locale, "error.cant_connect"));
    }

    // Initialize the player.
    if let Err(e) = manager
        .init(
            guild_id,
            interaction.channel_id,
            &interaction
                .guild_locale
                .clone()
                .unwrap_or(interaction.locale.clone()),
            template,
        )
        .await
    {
        event!(Level::ERROR, error = %e, "cannot initialize the player");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    }

    let template_name = match template {
        PlayerTemplate::Default => t(&interaction.locale, "join.template_default"),
        PlayerTemplate::Music => t(&interaction.locale, "join.template_music"),
        PlayerTemplate::Queue => t(&interaction.locale, "join.template_queue"),
        PlayerTemplate::Manual => t(&interaction.locale, "join.template_manual"),
        PlayerTemplate::Rpg => t(&interaction.locale, "join.template_rpg"),
        PlayerTemplate::Autoplay => t(&interaction.locale, "join.template_autoplay"),
    };

    let play_command = match LOADED_COMMANDS.get().and_then(|v| v.get("play")) {
        Some(v) => Cow::owned(format!("</play:{}>", v.get())),
        None => Cow::borrowed("`/play`"),
    };

    t_vars(
        &interaction.locale,
        "join.result",
        [template_name, play_command.as_ref()],
    )
}

/// Creates the `/join` [CreateCommand].
pub fn create_command() -> CreateCommand {
    let mut command = CreateCommand::new("join");

    command = serenity_command_name("join.name", command);
    command = serenity_command_description("join.description", command);

    command
        .description("Make me join your voice chat without playing anything.")
        .add_option({
            let mut option = CreateCommandOption::new(
                CommandOptionType::String,
                "template",
                "The template to create the player with.",
            )
            .required(false)
            .add_string_choice_localized("Default", "default", t_all("join.template_default"))
            .add_string_choice_localized("Music", "music", t_all("join.template_music"))
            .add_string_choice_localized("Queue", "queue", t_all("join.template_queue"))
            .add_string_choice_localized("Manual", "manual", t_all("join.template_manual"))
            .add_string_choice_localized("RPG", "rpg", t_all("join.template_rpg"))
            .add_string_choice_localized(
                "Autoplay",
                "autoplay",
                t_all("join.template_autoplay"),
            );

            option = serenity_command_option_name("join.template_name", option);
            option = serenity_command_option_description("join.template_description", option);

            option
        })
        .dm_permission(false)
}
