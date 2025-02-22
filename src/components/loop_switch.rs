//! 'loop' component execution.

use beef::lean::Cow;
use serenity::all::{ComponentInteraction, Context};
use tracing::{event, Level};

use crate::{
    i18n::{t, t_vars},
    music::LoopMode,
    utils, PLAYER_MANAGER,
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

    let voice_channel_id =
        match utils::get_voice_channel(context, &interaction.locale, guild_id, interaction.user.id)
        {
            Ok(v) => v,
            Err(e) => return e,
        };

    let player_state = manager
        .get_voice_channel_id(guild_id)
        .zip(manager.get_loop_mode(guild_id));

    if let Some((my_channel_id, current_loop_mode)) = player_state {
        if my_channel_id == voice_channel_id {
            let new_loop_mode = current_loop_mode.next();

            manager.set_loop_mode(guild_id, new_loop_mode).await;

            let loop_type_translation_key = match new_loop_mode {
                LoopMode::None => "loop.normal",
                LoopMode::Autopause => "loop.pause",
                LoopMode::Single => "loop.music",
                LoopMode::All => "loop.queue",
            };

            Cow::borrowed(t(&interaction.locale, loop_type_translation_key))
        } else {
            Cow::borrowed(t(&interaction.locale, "error.not_in_voice_channel"))
        }
    } else {
        Cow::borrowed(t(&interaction.locale, "error.player_not_exists"))
    }
}
