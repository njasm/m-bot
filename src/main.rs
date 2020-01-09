#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;

use chrono::{NaiveTime, Timelike};
use serenity::{
    Client,
    model::{channel::Message, gateway::Ready, event::ResumedEvent},
    prelude::*,
    Result as SerenityResult,
};

struct Handler;

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {

        if msg.author.bot {
            return;
        }

        debug!("Shard : {:?}", ctx.shard_id);
        debug!("Message received: {:?}", msg);

        //if msg.author.id == 
        if msg.content.starts_with("!ping") {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            check_sending_message(msg.channel_id.say(&ctx.http, "Pong!"))
        }

        if msg.content.starts_with("!time") {
            let mut iter = msg.content.split_whitespace();
            let _z = iter.next(); // first values is the command "!time"
            if let Some(v) = iter.next() {
                match v.parse::<i32>() {
                    Ok(mut int_value) => {
                        //let mut my_int = int_value;

                        // safeguard zero or below
                        if int_value < 0 {
                            check_sending_message(msg.channel_id.say(&ctx.http, "Supplied argument for seconds is two low!"));
                            return;
                        }

                        // safeguard max 3600
                        if int_value >3600 {
                            check_sending_message(msg.channel_id.say(&ctx.http, "Supplied argument for seconds is above max of 3600!"));
                            return;
                        }


                        while int_value >= 0 {
                            let t = NaiveTime::from_num_seconds_from_midnight(int_value as u32, 0);
                            let send_text = format!("COUNTDOWN: {} sec(s) - ({}h{}m{}s)", int_value, t.hour(), t.minute(), t.second());
                            check_sending_message(msg.channel_id.say(&ctx.http, send_text));

                            if int_value == 0 {
                                let boom = vec!["D:/Code/rust/m-bot/resources/gif/boom.gif"];
                                check_sending_message(msg.channel_id.send_files(&ctx.http, boom, |m| { m.content("") }));
                                return;
                            }

                            int_value = int_value - 1;
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                        }
                    },
                    Err(_) => check_sending_message(msg.channel_id.say(&ctx.http, "Supplied argument for seconds must be present and an integer number above zero."))
                }
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    fn ready(&self, _: Context, ready: Ready) {
        debug!("{} is connected!", ready.user.name);
    }

    fn resume(&self, _: Context, resume: ResumedEvent) {
        // Log at the DEBUG level.
        //
        // In this example, this will not show up in the logs because DEBUG is
        // below INFO, which is the set debug level.
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

fn main() {

    std::env::set_var("RUST_LOG", "m_bot");
    env_logger::init();
    info!("Hello, world!");

    // bot token
    let token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN ENV VAR NOT FOUND");

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

fn check_sending_message(result: SerenityResult<Message>) {
    // Sending a message can fail, due to a network error, an
    // authentication error, or lack of permissions to post in the
    // channel, so log to stdout when some error happens, with a
    // description of it.
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}