//! This module contains the shared behavior for the `shuffle` command and component.

use crate::i18n::t;
use crate::shared::SharedInteraction;
use crate::utils::player_not_exists;
use crate::{PLAYER_MANAGER, utils};
use beef::lean::Cow;
use serenity::all::Context;
use tracing::{Level, event};

/// Executes the `shuffle` shared behavior.
pub async fn execute<'a>(context: &Context, interaction: &SharedInteraction<'_>) -> Cow<'a, str> {
    let Some(guild_id) = interaction.guild_id() else {
        event!(Level::WARN, "interaction.guild_id is None");
        return Cow::borrowed(t(interaction.locale(), "error.not_in_guild"));
    };

    let Some(manager) = PLAYER_MANAGER.get() else {
        event!(Level::ERROR, "PLAYER_MANAGER.get() returned None");
        return Cow::borrowed(t(interaction.locale(), "error.unknown"));
    };

    let voice_channel_id = match utils::get_voice_channel(
        context,
        interaction.locale(),
        guild_id,
        interaction.user().id,
    ) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let player_state = manager.get_voice_channel_id(guild_id).await;

    if let Some(my_channel_id) = player_state {
        if my_channel_id == voice_channel_id {
            if let Err(e) = manager.shuffle(guild_id) {
                event!(Level::WARN, error = %e, "cannot shuffle queue");

                player_not_exists(context, interaction).await
            } else {
                Cow::borrowed(t(interaction.locale(), "shuffle.result"))
            }
        } else {
            Cow::borrowed(t(interaction.locale(), "error.not_in_voice_channel"))
        }
    } else {
        player_not_exists(context, interaction).await
    }
}
