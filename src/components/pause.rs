//! 'pause' component execution.

use beef::lean::Cow;
use serenity::all::{ComponentInteraction, Context};
use tracing::{event, Level};

use crate::i18n::t;
use crate::utils::delete_player_message;
use crate::{utils, PLAYER_MANAGER};

/// Executes the `pause` command.
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
        .zip(manager.get_pause(guild_id));

    if let Some((my_channel_id, paused)) = player_state {
        if my_channel_id == voice_channel_id {
            let new_paused = !paused;

            if let Err(e) = manager.set_pause(guild_id, new_paused).await {
                event!(Level::ERROR, error = ?e, pause = new_paused, "cannot resume/pause the player");
                return Cow::borrowed(t(&interaction.locale, "error.unknown"));
            }

            let translation_key = if new_paused {
                "pause.paused"
            } else {
                "pause.resumed"
            };

            Cow::borrowed(t(&interaction.locale, translation_key))
        } else {
            Cow::borrowed(t(&interaction.locale, "error.not_in_voice_channel"))
        }
    } else {
        delete_player_message(context, interaction).await;

        Cow::borrowed(t(&interaction.locale, "error.player_not_exists"))
    }
}
