//! Hydrogen // Components
//!
//! This module contains all the components from Hydrogen.

use beef::lean::Cow;
use serenity::all::{ComponentInteraction, Context};
use tracing::error;

mod loop_switch;
mod pause;
mod prev;
mod shuffle;
mod skip;
mod stop;

pub async fn execute<'a>(
    context: &Context,
    component: &ComponentInteraction,
) -> Option<Cow<'a, str>> {
    Some(match component.data.custom_id.as_str() {
        "loop" => loop_switch::execute(context, component).await,
        "pause" => pause::execute(context, component).await,
        "prev" => prev::execute(context, component).await,
        "skip" => skip::execute(context, component).await,
        "stop" => stop::execute(context, component).await,
        "shuffle" => shuffle::execute(context, component).await,
        _ => {
            error!(
                "(components::execute): unknown component: {}",
                component.data.custom_id
            );
            return None;
        }
    })
}
