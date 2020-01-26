extern crate reqwest;

use crate as bot;
use bot::tts::{AzureTextToSpeech, TextToSpeech, VoiceRSS};
use bot::VoiceManager;

use chrono::{NaiveTime, Timelike};
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::time::Instant;

use serenity::utils::content_safe as serenity_util_content_safe;
use serenity::utils::ContentSafeOptions;

use serenity::voice::pcm;

#[command]
fn vsay(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

            return Ok(());
        }
    };

    info!("ARGS: {:?}", args);
    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    let handler = match manager.get_mut(guild_id) {
        Some(handler) => handler,
        None => {
            bot::check_sending_message(msg.reply(&ctx, "Not in a voice channel"));

            return Ok(());
        }
    };

    // GETING AUDIO FROM VOICERSS.ORG API
    let settings = if let Some(guild_id) = msg.guild_id {
        // By default roles, users, and channel mentions are cleaned.
        ContentSafeOptions::default()
            .clean_channel(false)
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let voicerss_token = std::env::var("VOICERSS_TOKEN").expect("VOICE RSS TOKEN");
    let content = serenity_util_content_safe(&ctx.cache, &args.rest(), &settings);
    info!("CONTENT: {}", content);
    let url = format!(
        "http://api.voicerss.org/?key={}&c=wav&f=48Khz_16bit_stereo&r=4&hl=en-us&b64=false&src={}",
        voicerss_token, content
    );
    info!("URL: {}", url);
    let r = match reqwest::blocking::get(url.as_str()) {
        Ok(r) => r,
        Err(_) => {
            bot::check_sending_message(msg.reply(&ctx, "Unable to create the vocalization."));

            return Ok(());
        }
    };

    handler.play(pcm(true, r));

    Ok(())
}

#[command]
fn vtime(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

            return Ok(());
        }
    };

    info!("ARGS: {:?}", args);
    if let Some(seconds) = args.current() {
        match seconds.parse::<i32>() {
            Err(_) => {
                crate::check_sending_message(msg.channel_id.say(&ctx.http, "Supplied argument for seconds must be present and an integer number above zero."));

                return Err(CommandError(String::from("Supplied argument for seconds must be present and an integer number above zero.")));
            }
            Ok(mut int_value) => {
                // safeguard zero or below
                if int_value < 0 {
                    crate::check_sending_message(
                        msg.channel_id
                            .say(&ctx.http, "Supplied argument for seconds is two low!"),
                    );

                    return Err(CommandError(String::from(
                        "Supplied argument for seconds is two low!",
                    )));
                }

                // safeguard max 3600
                if int_value > 3600 {
                    crate::check_sending_message(msg.channel_id.say(
                        &ctx.http,
                        "Supplied argument for seconds is above max of 3600!",
                    ));

                    return Err(CommandError(String::from(
                        "Supplied argument for seconds is above max of 3600!",
                    )));
                }

                let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().unwrap();
                let mut manager = manager_lock.lock();

                let handler = match manager.get_mut(guild_id) {
                    Some(handler) => handler,
                    None => {
                        bot::check_sending_message(msg.reply(&ctx, "Not in a voice channel"));

                        return Ok(());
                    }
                };

                // let mut service = VoiceRSS::default();
                let mut service = AzureTextToSpeech::default();
                while int_value >= 0 {
                    let now = Instant::now();
                    let t = NaiveTime::from_num_seconds_from_midnight(int_value as u32, 0);

                    let the_text;
                    if t.minute() > 0 {
                        the_text = format!("{}m{}s", t.minute(), t.second());
                    } else {
                        the_text = format!("{}", t.second());
                    }

                    let r = match service.get_speech(the_text.as_str()) {
                        Ok(r) => r,
                        Err(_) => {
                            bot::check_sending_message(
                                msg.reply(&ctx, "Unable to create the vocalization."),
                            );

                            return Ok(());
                        }
                    };

                    while now.elapsed().as_millis() < 950 {
                        std::thread::sleep(std::time::Duration::from_millis(25));
                    }

                    int_value = int_value - 1;
                    //let _safe_audio: LockedAudio = handler.play_only(pcm(true, r));
                    handler.play(pcm(false, r));
                }
            }
        }
    }

    Ok(())
}

#[command]
#[description("Join a voice channel that you are also connected to.")]
#[num_args(0)]
fn join(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild = match msg.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

            return Ok(());
        }
    };

    let guild_id = guild.read().id;

    let channel_id = guild
        .read()
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            bot::check_sending_message(msg.reply(&ctx, "Not in a voice channel"));

            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    //if manager.join(guild_id, connect_to).is_some() {
    if let Some(handler) = manager.join(guild_id, connect_to) {
        //handler.listen(Some(Box::new(bot::Receiver::new())));
        bot::check_sending_message(
            msg.channel_id
                .say(&ctx.http, &format!("Joined {}", connect_to.mention())),
        );
    } else {
        bot::check_sending_message(msg.channel_id.say(&ctx.http, "Error joining the channel"));
    }

    Ok(())
}

#[command]
fn leave(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
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
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);
        bot::check_sending_message(msg.channel_id.say(&ctx.http, "Left voice channel"));
    } else {
        bot::check_sending_message(msg.reply(&ctx, "Not in a voice channel"));
    }

    Ok(())
}

#[command]
fn mute(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
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
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    let handler = match manager.get_mut(guild_id) {
        Some(handler) => handler,
        None => {
            bot::check_sending_message(msg.reply(&ctx, "Not in a voice channel"));

            return Ok(());
        }
    };

    if handler.self_mute {
        bot::check_sending_message(msg.channel_id.say(&ctx.http, "Already muted"));
    } else {
        handler.mute(true);

        bot::check_sending_message(msg.channel_id.say(&ctx.http, "Now muted"));
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
fn unmute(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            bot::check_sending_message(msg.channel_id.say(&ctx.http, "Error finding channel info"));

            return Ok(());
        }
    };
    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.mute(false);
        bot::check_sending_message(msg.channel_id.say(&ctx.http, "Unmuted"));
    } else {
        bot::check_sending_message(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to unmute in"),
        );
    }

    Ok(())
}

#[command]
fn deafen(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            bot::check_sending_message(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    let handler = match manager.get_mut(guild_id) {
        Some(handler) => handler,
        None => {
            bot::check_sending_message(msg.reply(&ctx, "Not in a voice channel"));

            return Ok(());
        }
    };

    if handler.self_deaf {
        bot::check_sending_message(msg.channel_id.say(&ctx.http, "Already deafened"));
    } else {
        handler.deafen(true);

        bot::check_sending_message(msg.channel_id.say(&ctx.http, "Deafened"));
    }

    Ok(())
}

#[command]
fn undeafen(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            bot::check_sending_message(msg.channel_id.say(&ctx.http, "Error finding channel info"));

            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.deafen(false);
        bot::check_sending_message(msg.channel_id.say(&ctx.http, "Undeafened"));
    } else {
        bot::check_sending_message(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to undeafen in"),
        );
    }

    Ok(())
}
