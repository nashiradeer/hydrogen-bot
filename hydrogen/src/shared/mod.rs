//! Shared behavior and logic between commands and components.

use serenity::all::{CommandInteraction, ComponentInteraction, GuildId, Message, User};

pub mod pause;
pub mod prev;
pub mod shuffle;
pub mod skip;
pub mod stop;

/// A wrapper for both [ComponentInteraction] and [CommandInteraction].
pub enum SharedInteraction<'a> {
    /// Wraps a [ComponentInteraction].
    Component(&'a ComponentInteraction),
    /// Wraps a [CommandInteraction].
    Command(&'a CommandInteraction),
}

impl SharedInteraction<'_> {
    /// Gets the guild ID of the interaction.
    pub fn guild_id(&self) -> Option<GuildId> {
        match self {
            SharedInteraction::Component(i) => i.guild_id,
            SharedInteraction::Command(i) => i.guild_id,
        }
    }

    /// Gets the locale of the interaction.
    pub fn locale(&self) -> &str {
        match self {
            SharedInteraction::Component(i) => &i.locale,
            SharedInteraction::Command(i) => &i.locale,
        }
    }

    /// Gets the user of the interaction.
    pub fn user(&self) -> &User {
        match self {
            SharedInteraction::Component(i) => &i.user,
            SharedInteraction::Command(i) => &i.user,
        }
    }

    /// Gets the message of the interaction if it is a [ComponentInteraction].
    pub fn message(&self) -> Option<&Message> {
        match self {
            SharedInteraction::Component(i) => Some(&i.message),
            SharedInteraction::Command(_) => None,
        }
    }
}

impl<'a> From<&'a ComponentInteraction> for SharedInteraction<'a> {
    fn from(value: &'a ComponentInteraction) -> Self {
        Self::Component(value)
    }
}

impl<'a> From<&'a CommandInteraction> for SharedInteraction<'a> {
    fn from(value: &'a CommandInteraction) -> Self {
        Self::Command(value)
    }
}
