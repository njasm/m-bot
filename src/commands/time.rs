use chrono::{NaiveTime, Timelike};
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    CommandError,
    macros::command,
};

#[command]
fn time(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut iter = msg.content.split_whitespace();
    let _z = iter.next(); // first values is the command "!time"
    if let Some(v) = iter.next() {
        match v.parse::<i32>() {
            Ok(mut int_value) => {

                // safeguard zero or below
                if int_value < 0 {
                    crate::check_sending_message(msg.channel_id.say(&ctx.http, "Supplied argument for seconds is two low!"));

                    return Err(CommandError(String::from("Supplied argument for seconds is two low!")));
                }

                // safeguard max 3600
                if int_value >3600 {
                    crate::check_sending_message(msg.channel_id.say(&ctx.http, "Supplied argument for seconds is above max of 3600!"));
                    
                    return Err(CommandError(String::from("Supplied argument for seconds is above max of 3600!")));
                }

                while int_value >= 0 {
                    let t = NaiveTime::from_num_seconds_from_midnight(int_value as u32, 0);
                    let send_text = format!("COUNTDOWN: {} sec(s) - ({}h{}m{}s)", int_value, t.hour(), t.minute(), t.second());
                    crate::check_sending_message(msg.channel_id.say(&ctx.http, send_text));

                    if int_value == 0 {
                        let boom = vec!["D:/Code/rust/m-bot/resources/gif/boom.gif"];
                        crate::check_sending_message(msg.channel_id.send_files(&ctx.http, boom, |m| { m.content("") }));

                        return Ok(());
                    }

                    int_value = int_value - 1;
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }
            ,
            Err(_) => {
                crate::check_sending_message(msg.channel_id.say(&ctx.http, "Supplied argument for seconds must be present and an integer number above zero."));

                return Err(CommandError(String::from("Supplied argument for seconds must be present and an integer number above zero.")));
            }
        }
    }

    Ok(())
}