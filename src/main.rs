#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod commands;
mod tts;

#[macro_use]
extern crate log;
extern crate chrono;
extern crate env_logger;
extern crate serenity;

use serenity::{
    client::bridge::{gateway::ShardManager, voice::ClientVoiceManager},
    framework::standard::{
        help_commands,
        macros::{check, command, group, help},
        Args, CommandGroup, CommandResult, DispatchError, HelpOptions, StandardFramework,
    },
    model::{channel::Message, event::ResumedEvent, gateway::Ready, id::GuildId, id::UserId},
    prelude::*,
    voice::AudioReceiver,
    Client, Result as SerenityResult,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

struct RollCallManager {
    list: HashMap<GuildId, RollCall>,
}

impl TypeMapKey for RollCallManager {
    type Value = Arc<Mutex<RollCallManager>>;
}

impl RollCallManager {
    fn new() -> Self {
        Self {
            list: HashMap::new(),
        }
    }

    fn start_roll_call_for(&mut self, guild_id: GuildId, call_by: UserId, requested: u16) -> bool {
        let c = guild_id.clone();
        if self.have_running_call_for(c) {
            false
        } else {
            self.list
                .insert(guild_id, RollCall::new(guild_id, call_by, requested));
            true
        }
    }

    // returns true if roll call was found for this guild and removed successfully, false otherwise
    fn cancel_running_call_for(&mut self, guild_id: GuildId) -> bool {
        match self.list.remove(&guild_id) {
            Some(_) => true,
            None => false,
        }
    }

    fn have_running_call_for(&self, guild_id: GuildId) -> bool {
        self.list.contains_key(&guild_id)
    }

    /// Returns true if user is joined to the roll call, false otherwise
    fn join_user_to_call(&mut self, guild_id: GuildId, user_id: UserId) -> bool {
        if self.have_running_call_for(guild_id) {
            let result = match self.list.get_mut(&guild_id) {
                Some(rc) => rc.join_user(user_id),
                None => false,
            };

            return result;
        }

        false
    }

    fn get_roll_call_for(&self, guild_id: GuildId) -> Option<&RollCall> {
        self.list.get(&guild_id)
    }
}

struct RollCall {
    guild_id: GuildId,
    call_by: UserId,
    requested: u16,
    joined: HashSet<UserId>,
}

impl RollCall {
    fn new(guild_id: GuildId, call_by: UserId, requested: u16) -> Self {
        Self {
            guild_id: guild_id,
            call_by: call_by,
            requested: requested,
            joined: HashSet::<UserId>::new(),
        }
    }

    fn complete(&self) -> bool {
        self.joined.len() == self.requested as usize
    }

    fn lack(&self) -> u16 {
        use std::convert::TryFrom;
        let r = usize::try_from(self.requested).unwrap() - self.joined.len();

        u16::try_from(r).unwrap()
    }

    fn has_user_joined(&self, user_id: UserId) -> bool {
        self.joined.contains(&user_id)
    }

    fn join_user(&mut self, user_id: UserId) -> bool {
        self.joined.insert(user_id)
    }
}

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

use commands::{ping::*, roll_call::*, say::*, shard::*, time::*, voice::*};

group!({
    name: "general",
    options: {},
    commands: [ping, say, time, latency],
});

group!({
    name: "Voice",
    options: {},
    commands: [join, leave, mute, unmute, deafen, undeafen, vtime, vsay],
});

group!({
    name: "Rally",
    options: {
        prefix: "rc",
        description: "Manage a Roll Call for your team's next rally."
    },
    commands: [start, ready, cancel, status],
});

#[help]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[wrong_channel = "Strike"]
#[embed_success_colour(BLUE)]
#[embed_error_colour(RED)]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

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

// Audio Receiver
struct Receiver;
impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        Self {}
    }
}

impl AudioReceiver for Receiver {
    fn speaking_update(&mut self, _ssrc: u32, _user_id: u64, _speaking: bool) {
        // You can implement logic here so that you can differentiate users'
        // SSRCs and map the SSRC to the User ID and maintain a state in
        // `Receiver`. Using this map, you can map the `ssrc` in `voice_packet`
        // to the user ID and handle their audio packets separately.
        info!(
            "[AUDIO SPEAKING UPDATE] SSRC: {}, UserID: {}, Speeking: {}",
            _ssrc, _user_id, _speaking
        );
    }

    fn voice_packet(
        &mut self,
        ssrc: u32,
        sequence: u16,
        _timestamp: u32,
        _stereo: bool,
        data: &[i16],
        compressed_size: usize,
    ) {
        info!("Audio packet's first 5 bytes: {:?}", data.get(..5));
        info!(
            "Audio packet sequence {:05} has {:04} bytes (decompressed from {}), SSRC {}",
            sequence,
            data.len(),
            compressed_size,
            ssrc,
        );
    }

    fn client_connect(&mut self, _ssrc: u32, _user_id: u64) {
        // You can implement your own logic here to handle a user who has joined the
        // voice channel e.g., allocate structures, map their SSRC to User ID.
        info!(
            "[AUDIO CLIENT CONNECT] SSRC: {}, UserID: {}",
            _ssrc, _user_id
        );
    }

    fn client_disconnect(&mut self, _user_id: u64) {
        // You can implement your own logic here to handle a user who has left the
        // voice channel e.g., finalise processing of statistics etc.
        // You will typically need to map the User ID to their SSRC; observed when
        // speaking or connecting.
        info!("[AUDIO CLIENT DISCONNECT] UserID: {}", _user_id);
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "m_bot=trace,serenity=trace");
    std::env::set_var("RUST_BACKTRACE", "1");
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
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<RollCallManager>(Arc::new(Mutex::new(RollCallManager::new())));
    }

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    client.with_framework(
        // Configures the client, allowing for options to mutate how the
        // framework functions.
        //
        // Refer to the documentation for
        // `serenity::ext::framework::Configuration` for all available
        // configurations.
        StandardFramework::new()
            .configure(|c| {
                c.with_whitespace(true)
                    .on_mention(Some(bot_id))
                    .prefix(".")
                    .case_insensitivity(true)
                    // Sets the bot's owners. These will be used for commands that
                    // are owners only.
                    .owners(owners)
            })
            // Set a function that's called whenever an attempted command-call's
            // command could not be found.
            .unrecognised_command(|_, _, unknown_command_name| {
                println!("Could not find command named '{}'", unknown_command_name);
            })
            // Set a function that's called whenever a message is not a command.
            .normal_message(|ctx, msg| {
                let guild_lock = match msg.guild(&ctx.cache) {
                    Some(g) => g,
                    None => return (),
                };

                let user_id = msg.author.id;
                let user_name = &msg.author.name;

                let guild = guild_lock.read();
                let guild_id = guild.id;
                let guild_name = &guild.name;

                let message = format!(
                    "Message ({} - {}) ({} - {}): {}",
                    guild_id, guild_name, user_id, user_name, msg.content
                );
                info!("{}", message);
            })
            // Set a function that's called whenever a command's execution didn't complete for one
            // reason or another. For example, when a user has exceeded a rate-limit or a command
            // can only be performed by the bot owner.
            .on_dispatch_error(|ctx, msg, error| {
                if let DispatchError::Ratelimited(seconds) = error {
                    let _ = msg.channel_id.say(
                        &ctx.http,
                        &format!("Try this again in {} seconds.", seconds),
                    );
                }
            })
            // The `#[group]` macro generates `static` instances of the options set for the group.
            // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
            // #name is turned all uppercase
            .group(&GENERAL_GROUP)
            .group(&VOICE_GROUP)
            .group(&RALLY_GROUP)
            .help(&MY_HELP),
    );

    // lets create the http server for azure does not kill our server.
    let server = tiny_http::Server::http("0.0.0.0:80").unwrap();
    std::thread::spawn(move || {
        info!("Starting tiny http server!");
        loop {
            // blocks until the next request is received
            let request = match server.recv() {
                Ok(rq) => rq,
                Err(e) => {
                    println!("error: {}", e);
                    break;
                }
            };

            let _ = request.respond(tiny_http::Response::from_string("Hello, m-bot running!"));
        }
    });

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}

/// Sending a message can fail, due to a network error, an
/// authentication error, or lack of permissions to post in the
/// channel, so log to stdout when some error happens, with a
/// description of it.
fn check_sending_message(result: SerenityResult<Message>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}
