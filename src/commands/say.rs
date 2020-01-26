use crate as bot;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::utils::ContentSafeOptions;
use serenity::utils::content_safe as serenity_util_content_safe;
use serenity::framework::standard::{
    CommandResult, 
    macros::command,
    Args,
};

#[command]
fn say(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
       // By default roles, users, and channel mentions are cleaned.
       ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // If it's a guild channel, we want mentioned users to be displayed
            // as their display name.
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let content = serenity_util_content_safe(&ctx.cache, &args.rest(), &settings);
    bot::check_sending_message(msg.channel_id.say(&ctx.http, &content));

    Ok(())
}