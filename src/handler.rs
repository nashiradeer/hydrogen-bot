//! Command and component handler for Hydrogen.

use std::{collections::HashMap, time::Duration};

use serenity::{
    all::{ChannelId, Command, CommandInteraction, ComponentInteraction, UserId},
    builder::{CreateEmbed, CreateEmbedFooter, EditInteractionResponse},
    client::Context,
    http::{CacheHttp, Http},
};
use tokio::{spawn, time::sleep};
use tracing::{debug, error, info, warn};

use crate::{
    commands, components,
    i18n::t,
    utils::constants::{HYDROGEN_ERROR_COLOR, HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR},
    COMPONENTS_MESSAGES, LOADED_COMMANDS,
};

/// Type used to monitor the responses sent by the bot.
pub type AutoRemoverKey = (ChannelId, UserId);

/// Handles a command interaction.
pub async fn handle_command(context: &Context, command: &CommandInteraction) {
    if let Err(e) = command.defer_ephemeral(&context.http).await {
        error!("(handle_command): failed to defer interaction: {}", e);
        return;
    }

    let response = commands::execute(context, command).await;

    if let Some(message) =
        response.map(|response| response.into_edit_interaction_response(&command.locale))
    {
        if let Err(e) = command.edit_response(&context.http, message).await {
            error!("(handle_command): cannot respond to the interaction: {}", e);
        }
    }
}

/// Handles a component interaction.
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
}

/// Registers the commands.
pub async fn register_commands(http: impl AsRef<Http>) -> bool {
    let commands = commands::all_create_commands();

    debug!(
        "(register_command): registering {} commands...",
        commands.len()
    );

    match Command::set_global_commands(http, commands.to_vec()).await {
        Ok(v) => {
            info!("(register_command): registered {} commands", v.len());

            let mut commands_id = HashMap::new();

            for commands in v {
                commands_id.insert(commands.name.clone(), commands.id);
            }

            LOADED_COMMANDS
                .set(commands_id)
                .expect("cannot set the loaded commands");

            true
        }
        Err(e) => {
            error!("(register_command): cannot register the commands: {}", e);

            false
        }
    }
}

/// Removes the response after a certain time.
async fn autoremover(key: AutoRemoverKey) {
    sleep(Duration::from_secs(10)).await;
    debug!("(autoremover): removing response {:?} from cache...", key);
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

impl<'a> ResponseValue<'a> {
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
