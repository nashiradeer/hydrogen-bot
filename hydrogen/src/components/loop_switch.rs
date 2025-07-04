//! 'loop' component execution.

use beef::lean::Cow;
use serenity::all::{ComponentInteraction, Context};
use tracing::{Level, event};

use crate::shared::SharedInteraction;
use crate::utils::delete_player_message;
use crate::{PLAYER_MANAGER, i18n::t, music::LoopMode, utils};

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
        .await
        .zip(manager.get_loop_mode(guild_id));

    if let Some((my_channel_id, current_loop_mode)) = player_state {
        if my_channel_id == voice_channel_id {
            let new_loop_mode = current_loop_mode.next();

            manager.set_loop_mode(guild_id, new_loop_mode).await;

            let loop_type_translation_key = match new_loop_mode {
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
