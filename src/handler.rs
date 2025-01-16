//! Command and component handler for Hydrogen.

use std::{collections::HashMap, time::Duration};

use serenity::{
    all::{ChannelId, Command, CommandInteraction, UserId},
    builder::{CreateEmbed, CreateEmbedFooter, EditInteractionResponse},
    client::Context,
    http::Http,
};
use tokio::time::sleep;
use tracing::{event, instrument, Level};

use crate::{
    commands,
    i18n::t,
    utils::constants::{HYDROGEN_ERROR_COLOR, HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR},
    COMPONENTS_MESSAGES, LOADED_COMMANDS,
};

/// Type used to monitor the responses sent by the bot.
pub type AutoRemoverKey = (ChannelId, UserId);

/// Handles a command interaction.
#[instrument(skip_all, name = "command_handler", fields(command_name = %command.data.name, user_id = %command.user.id, guild_id = ?command.guild_id.map(|v| v.get()), channel_id = %command.channel_id))]
pub async fn handle_command(context: &Context, command: &CommandInteraction) {
    if let Err(e) = command.defer_ephemeral(&context.http).await {
        event!(Level::ERROR, error = ?e, "failed to defer interaction");
        return;
    }

    let response = commands::execute(context, command).await;

    if let Some(message) =
        response.map(|response| response.into_edit_interaction_response(&command.locale))
    {
        if let Err(e) = command.edit_response(&context.http, message).await {
            event!(
                Level::ERROR,
                error = ?e,
                "cannot edit the response"
            );
        }
    }
}

/* /// Handles a component interaction.
pub async fn handle_component(context: &Context, component: &ComponentInteraction) {
    if let Err(e) = component.defer_ephemeral(&context.http).await {
        error!("(handle_component): failed to defer interaction: {}", e);
        return;
    }

    let response = components::execute(context, component).await;

    if let Some(message) =
        response.map(|response| response.into_edit_interaction_response(&component.locale))
    {
        match component.edit_response(&context.http, message).await {
            Ok(v) => {
                let auto_remover_key = (v.channel_id, component.user.id);

                let auto_remover = spawn(async move {
                    autoremover(auto_remover_key).await;
                });

                if let Some((auto_remover, old_component)) =
                    COMPONENTS_MESSAGES.insert(auto_remover_key, (auto_remover, component.clone()))
                {
                    auto_remover.abort();

                    if let Err(e) = old_component.delete_response(context.http()).await {
                        warn!(
                            "(handle_component): cannot delete the message {:?}: {}",
                            auto_remover_key, e
                        );
                    }
                }
            }
            Err(e) => {
                error!(
                    "(handle_component): cannot respond to the interaction: {}",
                    e
                );
            }
        }
    }
} */

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

/// Removes the response after a certain time.
#[instrument(name = "message_autoremover")]
async fn autoremover(key: AutoRemoverKey) {
    sleep(Duration::from_secs(10)).await;
    event!(Level::DEBUG, message = ?key, "removing response from cache...");
    COMPONENTS_MESSAGES.remove(&key);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a response.
pub struct Response<'a> {
    /// The translation key for the title.
    title: ResponseValue<'a>,
    /// The translation key for the description.
    description: ResponseValue<'a>,
    /// The response type.
    pub response_type: ResponseType,
}

impl<'a> Response<'a> {
    pub fn new(title: &'a str, description: &'a str, response_type: ResponseType) -> Self {
        Self {
            title: ResponseValue::TranslationKey(title),
            description: ResponseValue::TranslationKey(description),
            response_type,
        }
    }

    pub fn raw(
        title: ResponseValue<'a>,
        description: ResponseValue<'a>,
        response_type: ResponseType,
    ) -> Self {
        Self {
            title,
            description,
            response_type,
        }
    }

    /// Converts the response into an embed.
    pub fn into_embed(self, lang: &str) -> CreateEmbed {
        CreateEmbed::new()
            .title(self.title.into_value(lang))
            .description(self.description.into_value(lang))
            .color(self.response_type.into_color())
            .footer(
                CreateEmbedFooter::new(t(lang, "generic.embed_footer")).icon_url(HYDROGEN_LOGO_URL),
            )
    }

    /// Converts the response into an [EditInteractionResponse].
    pub fn into_edit_interaction_response(self, lang: &str) -> EditInteractionResponse {
        EditInteractionResponse::new().embed(self.into_embed(lang))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a response value.
pub enum ResponseValue<'a> {
    /// Represents a translation key.
    TranslationKey(&'a str),

    #[allow(dead_code)]
    /// Represents a raw value.
    Raw(&'a str),

    /// Represents a raw string.
    RawString(String),
}

impl ResponseValue<'_> {
    /// Converts the [ResponseValue] into its value.
    pub fn into_value(self, lang: &str) -> String {
        match self {
            ResponseValue::TranslationKey(key) => t(lang, key).to_owned(),
            ResponseValue::Raw(value) => value.to_owned(),
            ResponseValue::RawString(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Represents a response type.
pub enum ResponseType {
    /// Represents a successful response.
    Success,
    /// Represents an error response.
    Error,
}

impl ResponseType {
    /// Converts the response type into a color.
    pub fn into_color(self) -> i32 {
        match self {
            ResponseType::Success => HYDROGEN_PRIMARY_COLOR,
            ResponseType::Error => HYDROGEN_ERROR_COLOR,
        }
    }
}
