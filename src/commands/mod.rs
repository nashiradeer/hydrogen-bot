//! Controls the command execution flow.

use serenity::all::{CommandInteraction, Context, CreateCommand};
use tracing::error;

use crate::handler::Response;

mod join;
mod play;
mod seek;

pub async fn execute<'a>(context: &Context, command: &CommandInteraction) -> Option<Response<'a>> {
    Some(match command.data.name.as_str() {
        "join" => join::execute(context, command).await,
        "seek" => seek::execute(context, command).await,
        "play" => play::execute(context, command).await,
        _ => {
            error!(
                "(commands::execute): unknown command: {}",
                command.data.name
            );
            return None;
        }
    })
}

pub fn all_create_commands<'a>() -> [CreateCommand; 3] {
    [
        join::create_command(),
        seek::create_command(),
        play::create_command(),
    ]
}
