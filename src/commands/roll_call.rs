use crate as bot;
use bot::RollCallManager;

// use std::time::Instant;
// use chrono::{NaiveTime, Timelike};
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[num_args(1)]
#[description("Start a Roll Call if one is not currently active.")]
#[example("start 10")]
#[aliases(start)]
#[only_in(guilds)]
pub fn start(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let requested_player_num = args.parse::<u16>()?;
    let guild_id = match msg.guild_id {
        Some(guild) => guild,
        None => {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<RollCallManager>()
        .cloned()
        .expect("Expected RollCallManager in ShareMap.");
    {
        let mut manager = manager_lock.lock();
        if manager.have_running_call_for(guild_id) {
            bot::check_sending_message(msg.channel_id.say(
                &ctx.http,
                "A Roll-Call is currently running. You need to cancel that one first.",
            ));

            return Ok(());
        }

        if manager.start_roll_call_for(guild_id, msg.author.id, requested_player_num) {
            let message = format!("@here, A Roll-Call was activated by <@{}>!\nIt is requested that {} players join it! Be the first.", msg.author.id, requested_player_num);
            bot::check_sending_message(msg.channel_id.say(&ctx.http, message));
        }

        Ok(())
    }
}

#[command]
#[only_in(guilds)]
#[description("Sets you ready by joining you in the Roll Call")]
#[aliases(ready)]
pub fn ready(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(guild) => guild,
        None => {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

            return Ok(());
        }
    };

    use crate::RollCallManager;
    let manager_lock = ctx
        .data
        .read()
        .get::<RollCallManager>()
        .cloned()
        .expect("Expected RollCallManager in ShareMap.");
    {
        let mut manager = manager_lock.lock();
        if !manager.have_running_call_for(guild_id) {
            bot::check_sending_message(
                msg.reply(&ctx, "There's no currently active Roll Call to join."),
            );

            return Ok(());
        }

        let joined = manager.join_user_to_call(guild_id, msg.author.id);
        if joined == true {
            bot::check_sending_message(msg.reply(&ctx, "You're ready!!"));

            let left = manager.get_roll_call_for(guild_id).unwrap().lack();
            let message = if left == 0 {
                format!("@here, Roll Call complete!!! BURNNNNN!!!!")
            } else {
                format!("@here, {} players left!", left)
            };

            if left == 0 {
                manager.cancel_running_call_for(guild_id);
            }

            bot::check_sending_message(msg.channel_id.say(&ctx.http, message));
        } else {
            // if we got here is because user is already joined.
            bot::check_sending_message(msg.reply(&ctx, "You already joined. relax!"));
        }
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[description("Cancels the active Roll Call")]
#[aliases(cancel)]
pub fn cancel(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(guild) => guild,
        None => {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<RollCallManager>()
        .cloned()
        .expect("Expected RollCallManager in ShareMap.");
    {
        let mut manager = manager_lock.lock();
        if manager.cancel_running_call_for(guild_id) {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "@here Roll-Call cancelled. :'("),
            );
        }

        Ok(())
    }
}

#[command]
#[only_in(guilds)]
#[description("Current Roll Call status")]
#[aliases(status)]
pub fn status(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(guild) => guild,
        None => {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<RollCallManager>()
        .cloned()
        .expect("Expected RollCallManager in ShareMap.");
    {
        let manager = manager_lock.lock();
        if !manager.have_running_call_for(guild_id) {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "There's no active Roll-Call. Start one first"),
            );

            return Ok(());
        }

        let rc = manager.get_roll_call_for(guild_id).unwrap();
        let mut message_builder = serenity::utils::MessageBuilder::new();
        message_builder
            .push_bold_line("Roll Call Status")
            .push_italic("Started by:")
            .push_line(format!(" {}", rc.call_by.mention()))
            .push_italic("Players Requested:")
            .push_line(format!(" {}", rc.requested))
            .push_italic("Players Joined:")
            .push_line(format!(" {}", rc.joined.len()));

        for v in &rc.joined {
            message_builder.push_line(format!("{} ", v.mention()));
        }

        let message = message_builder
            .push_italic("Players missing:")
            .push_line(format!(" {}", rc.lack()))
            .push_line("@here")
            .build();

        bot::check_sending_message(msg.channel_id.say(&ctx.http, message));
    }

    Ok(())
}
