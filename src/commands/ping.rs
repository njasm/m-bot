
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult, 
    macros::command,
};

#[command]
#[only_in(guilds)]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    crate::check_sending_message(msg.channel_id.say(&ctx.http, "Pong! : )"));

    Ok(())
}