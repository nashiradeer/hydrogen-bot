//! 'loop' component execution.

use beef::lean::Cow;
use serenity::all::{ComponentInteraction, Context};
use tracing::{event, Level};

use crate::{
    i18n::{t, t_vars},
    music::LoopMode,
    PLAYER_MANAGER,
};

/// Executes the `loop` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Cow<'a, str> {
    let Some(guild_id) = interaction.guild_id else {
        event!(Level::WARN, "interaction.guild_id is None");
        return Cow::borrowed(t(&interaction.locale, "error.not_in_guild"));
    };

    let Some(manager) = PLAYER_MANAGER.get() else {
        event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown"));
    };

    let Some(voice_channel_id) = context.cache.guild(guild_id).and_then(|guild| {
        guild
            .voice_states
            .get(&interaction.user.id)
            .and_then(|voice_state| voice_state.channel_id)
    }) else {
        event!(Level::INFO, "user voice state is None");
        return Cow::borrowed(t(&interaction.locale, "error.unknown_voice_state"));
    };

    let player_state = manager
        .get_voice_channel_id(guild_id)
        .zip(manager.get_loop_mode(guild_id));

    if let Some((my_channel_id, current_loop_mode)) = player_state {
        if my_channel_id == voice_channel_id {
            let new_loop_mode = current_loop_mode.next();

            manager.set_loop_mode(guild_id, new_loop_mode).await;

            let loop_type_translation_key = match new_loop_mode {
                LoopMode::None => "loop.autostart",
                LoopMode::Autopause => "loop.no_autostart",
                LoopMode::Single => "loop.music",
                LoopMode::All => "loop.queue",
            };

            let loop_type_translation = t(&interaction.locale, loop_type_translation_key);

            t_vars(&interaction.locale, "loop.looping", [loop_type_translation])
        } else {
            Cow::borrowed(t(&interaction.locale, "error.not_in_voice_channel"))
        }
    } else {
        Cow::borrowed(t(&interaction.locale, "error.player_not_exists"))
    }
}
