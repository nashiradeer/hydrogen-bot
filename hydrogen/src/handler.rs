//! Command and component handler for Hydrogen.

use beef::lean::Cow;
use moka::sync::Cache;
use serenity::all::{ChannelId, CreateInteractionResponseFollowup, Message};
use serenity::{
    all::{Command, CommandInteraction, ComponentInteraction, UserId},
    builder::EditInteractionResponse,
    client::Context,
    http::Http,
};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{event, instrument, Level};

use crate::{commands, components, LOADED_COMMANDS};

/// Cache of the messages used to clean up the old messages when too many messages are sent.
pub static MESSAGE_CACHE: LazyLock<Cache<(ChannelId, UserId), String>> = LazyLock::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(5))
        .build()
});

/// Handles a command interaction.
#[instrument(skip_all, name = "command_handler", fields(command_name = %command.data.name, user_id = %command.user.id, guild_id = ?command.guild_id.map(|v| v.get()), channel_id = %command.channel_id))]
pub async fn handle_command(context: &Context, command: &CommandInteraction) {
    let common = CommonInteraction::Command(command);

    let deferred = common.defer_ephemeral(&context.http).await;

    if let Some(message) = commands::execute(context, command).await {
        post_execute(context, deferred, message, &common).await;
    }
}

/// Handles a component interaction.
#[instrument(skip_all, name = "component_handler", fields(component_name = %component.data.custom_id, user_id = %component.user.id, guild_id = ?component.guild_id.map(|v| v.get()), channel_id = %component.channel_id))]
pub async fn handle_component(context: &Context, component: &ComponentInteraction) {
    let common = CommonInteraction::Component(component);

    let deferred = common.defer_ephemeral(&context.http).await;

    if let Some(message) = components::execute(context, component).await {
        post_execute(context, deferred, message, &common).await;
    }
}

/// Executed after the command or component execution.
async fn post_execute(
    context: &Context,
    deferred: bool,
    message: Cow<'_, str>,
    interaction: &CommonInteraction<'_>,
) {
    if let Some(old_message) =
        MESSAGE_CACHE.remove(&(interaction.channel_id(), interaction.user_id()))
    {
        if let Err(e) = context
            .http
            .delete_original_interaction_response(&old_message)
            .await
        {
            event!(Level::WARN, error = ?e, "cannot delete the old message");
        }
    }

    if deferred {
        match interaction
            .edit_response(
                &context.http,
                EditInteractionResponse::new().content(message.as_ref()),
            )
            .await
        {
            Ok(_) => {
                MESSAGE_CACHE.insert(
                    (interaction.channel_id(), interaction.user_id()),
                    interaction.token().to_owned(),
                );
                return;
            }
            Err(e) => {
                event!(Level::WARN, error = ?e, "cannot edit the response");
            }
        }
    }

    match interaction
        .create_followup(
            &context.http,
            CreateInteractionResponseFollowup::new().content(message.as_ref()),
        )
        .await
    {
        Ok(_) => {
            MESSAGE_CACHE.insert(
                (interaction.channel_id(), interaction.user_id()),
                interaction.token().to_owned(),
            );
        }
        Err(e) => {
            event!(
                Level::ERROR,
                error = ?e,
                "cannot create the response"
            );
        }
    }
}

/// Registers the commands.
pub async fn register_commands(http: impl AsRef<Http>) -> bool {
    let commands = commands::all_create_commands();

    event!(
        Level::DEBUG,
        commands_count = commands.len(),
        "registering commands..."
    );

    match Command::set_global_commands(http, commands.to_vec()).await {
        Ok(v) => {
            event!(Level::INFO, commands_count = v.len(), "registered commands");

            let mut commands_id = HashMap::new();

            for commands in v {
                commands_id.insert(commands.name.clone(), commands.id);
            }

            if LOADED_COMMANDS.set(commands_id).is_err() {
                event!(Level::WARN, "cannot set the loaded commands");
            }

            true
        }
        Err(e) => {
            event!(Level::ERROR, error = ?e, "cannot register the commands");

            false
        }
    }
}

/// A wrapper for command and component interactions for common operations.
enum CommonInteraction<'a> {
    /// Command interaction.
    Command(&'a CommandInteraction),
    /// Component interaction.
    Component(&'a ComponentInteraction),
}

impl CommonInteraction<'_> {
    /// Gets the user ID.
    fn user_id(&self) -> UserId {
        match self {
            Self::Command(v) => v.user.id,
            Self::Component(v) => v.user.id,
        }
    }

    /// Gets the channel ID.
    fn channel_id(&self) -> ChannelId {
        match self {
            Self::Command(v) => v.channel_id,
            Self::Component(v) => v.channel_id,
        }
    }

    /// Gets the token.
    fn token(&self) -> &str {
        match self {
            Self::Command(v) => &v.token,
            Self::Component(v) => &v.token,
        }
    }

    /// Defer the interaction.
    async fn defer_ephemeral(&self, http: &Http) -> bool {
        match self {
            Self::Command(v) => v.defer_ephemeral(http).await,
            Self::Component(v) => v.defer_ephemeral(http).await,
        }
        .inspect_err(|e| {
            event!(Level::WARN, error = ?e, "failed to defer interaction");
        })
        .is_ok()
    }

    /// Edit the response.
    async fn edit_response(
        &self,
        http: &Http,
        response: EditInteractionResponse,
    ) -> Result<Message, serenity::Error> {
        match self {
            Self::Command(v) => v.edit_response(http, response).await,
            Self::Component(v) => v.edit_response(http, response).await,
        }
    }

    /// Create a followup.
    async fn create_followup(
        &self,
        http: &Http,
        response: CreateInteractionResponseFollowup,
    ) -> Result<Message, serenity::Error> {
        match self {
            Self::Command(v) => v.create_followup(http, response).await,
            Self::Component(v) => v.create_followup(http, response).await,
        }
    }
}
