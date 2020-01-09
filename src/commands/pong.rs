use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::model::Mrd
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};

[#command(attr: TokenStream, input: TokenStream)]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.channel_id.say(&ctx.http, "Pong!");

    Ok(())
}