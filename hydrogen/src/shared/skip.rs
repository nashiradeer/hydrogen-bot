//! This module contains the shared behavior for the `skip` command and component.

use crate::i18n::{t, t_vars};
use crate::music::Track;
use crate::shared::SharedInteraction;
use crate::utils::delete_player_message;
use crate::{PLAYER_MANAGER, utils};
use beef::lean::Cow;
use serenity::all::Context;
use tracing::{Level, event};

/// Executes the `skip` shared behavior.
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

    let my_channel_id = manager.get_voice_channel_id(guild_id).await;

    if let Some(my_channel_id) = my_channel_id {
        if my_channel_id == voice_channel_id {
            let music = match manager.skip(guild_id).await {
                Ok(v) => v,
                Err(e) => {
                    event!(Level::ERROR, error = ?e, "cannot go to the next track");
                    return Cow::borrowed(t(interaction.locale(), "error.unknown"));
                }
            };

            let Some(music) = music else {
                return Cow::borrowed(t(interaction.locale(), "error.empty_queue"));
            };

            get_message(music, interaction)
        } else {
            Cow::borrowed(t(interaction.locale(), "error.not_in_voice_channel"))
        }
    } else {
        delete_player_message(context, interaction).await;

        Cow::borrowed(t(interaction.locale(), "error.player_not_exists"))
    }
}

/// Get the message to send to the user.
fn get_message<'a>(track: Track, interaction: &SharedInteraction<'_>) -> Cow<'a, str> {
    if let Some(uri) = track.url {
        t_vars(
            interaction.locale(),
            "skip.skipping_url",
            [track.title, track.author, uri],
        )
    } else {
        t_vars(
            interaction.locale(),
            "skip.skipping",
            [track.title, track.author],
        )
    }
}
