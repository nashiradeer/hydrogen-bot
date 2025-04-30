//! Controls the command execution flow.

use beef::lean::Cow;
use serenity::all::{CommandInteraction, Context, CreateCommand};
use tracing::{Level, event};

mod join;
mod loop_switch;
mod pause;
mod play;
mod prev;
mod shuffle;
mod skip;
mod stop;
mod time;

pub async fn execute<'a>(context: &Context, command: &CommandInteraction) -> Option<Cow<'a, str>> {
    Some(match command.data.name.as_str() {
        "skip" => skip::execute(context, command).await,
        "pause" => pause::execute(context, command).await,
        "previous" => prev::execute(context, command).await,
        "play" => play::execute(context, command).await,
        "loop" => loop_switch::execute(context, command).await,
        "shuffle" => shuffle::execute(context, command).await,
        "stop" => stop::execute(context, command).await,
        "join" => join::execute(context, command).await,
        "time" => time::execute(context, command).await,
        _ => {
            event!(Level::ERROR, "unknown command");
            return None;
        }
    })
}

pub fn all_create_commands() -> [CreateCommand; 9] {
    [
        skip::create_command(),
        pause::create_command(),
        prev::create_command(),
        play::create_command(),
        loop_switch::create_command(),
        shuffle::create_command(),
        stop::create_command(),
        join::create_command(),
        time::create_command(),
    ]
}
