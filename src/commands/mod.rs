//! Controls the command execution flow.

use beef::lean::Cow;
use serenity::all::{CommandInteraction, Context, CreateCommand};
use tracing::{event, Level};

mod join;
mod play;
mod time;

pub async fn execute<'a>(context: &Context, command: &CommandInteraction) -> Option<Cow<'a, str>> {
    Some(match command.data.name.as_str() {
        "join" => join::execute(context, command).await,
        "time" => time::execute(context, command).await,
        "play" => play::execute(context, command).await,
        _ => {
            event!(Level::ERROR, "unknown command");
            return None;
        }
    })
}

pub fn all_create_commands() -> [CreateCommand; 3] {
    [
        join::create_command(),
        time::create_command(),
        play::create_command(),
    ]
}
