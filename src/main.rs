mod commands;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate serenity;

use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use serenity::{
    Client,
    client::bridge::gateway::{ShardManager},
    framework::standard::{
        macros::group,
        DispatchError, StandardFramework, 
    },
    model::{channel::Message, gateway::Ready, event::ResumedEvent},
    prelude::*,
    Result as SerenityResult,
};

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

use commands::{
    ping::*,
    say::*,
    time::*,
};

group!({
    name: "general",
    options: {},
    commands: [ping, say, time],
  });
  


struct Handler;

impl EventHandler for Handler {
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
    
    {
        let mut data = client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
    
            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Commands are equivalent to:
    // "~about"
    // "~emoji cat"
    // "~emoji dog"
    // "~multiply"
    // "~ping"
    // "~some long command"
    client.with_framework(
        // Configures the client, allowing for options to mutate how the
        // framework functions.
        //
        // Refer to the documentation for
        // `serenity::ext::framework::Configuration` for all available
        // configurations.
        StandardFramework::new()
            .configure(|c| c
                .with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix(".")
                // Sets the bot's owners. These will be used for commands that
                // are owners only.
                .owners(owners)
            )
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        .unrecognised_command(|_, _, unknown_command_name| {
            println!("Could not find command named '{}'", unknown_command_name);
        })
        // Set a function that's called whenever a message is not a command.
        .normal_message(|_, message| {
            println!("Message is not a command '{}'", message.content);
        })
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(|ctx, msg, error| {
            if let DispatchError::Ratelimited(seconds) = error {
                let _ = msg.channel_id.say(&ctx.http, &format!("Try this again in {} seconds.", seconds));
            }
        })
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .group(&GENERAL_GROUP)
    );

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