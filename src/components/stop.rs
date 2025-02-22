//! 'stop' component execution.

use beef::lean::Cow;
use serenity::all::{ComponentInteraction, Context};
use tracing::{event, Level};

use crate::i18n::t;
use crate::{utils, PLAYER_MANAGER};

/// Executes the `stop` command.
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

    if let Some(my_channel_id) = manager.get_voice_channel_id(guild_id) {
        if my_channel_id == voice_channel_id {
            if let Err(e) = manager.destroy(guild_id).await {
                event!(Level::ERROR, error = ?e, "cannot stop the player");
                return Cow::borrowed(t(&interaction.locale, "error.unknown"));
            }

            Cow::borrowed(t(&interaction.locale, "stop.stopped"))
        } else {
            Cow::borrowed(t(&interaction.locale, "error.not_in_voice_channel"))
        }
    } else {
        Cow::borrowed(t(&interaction.locale, "error.player_not_exists"))
    }
}
