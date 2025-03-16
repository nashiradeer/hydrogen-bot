//! 'pause' component execution.

use beef::lean::Cow;
use serenity::all::{ComponentInteraction, Context};

use crate::shared;
use crate::shared::SharedInteraction;

/// Executes the `pause` command.
pub async fn execute<'a>(context: &Context, interaction: &ComponentInteraction) -> Cow<'a, str> {
    shared::pause::execute(context, &SharedInteraction::Component(interaction)).await
}
