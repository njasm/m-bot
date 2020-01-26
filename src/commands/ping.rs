use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult, 
    macros::command,
};

use serenity::http::AttachmentType;
use std::path::Path;
use serenity::utils::Colour;

#[command]
#[only_in(guilds)]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    crate::check_sending_message(msg.channel_id.say(&ctx.http, "Pong! : )"));

    let mm = msg.channel_id.send_message(&ctx.http, |m|  {
        m.content("Hello, World!");
        m.embed(|e| {
            e.title("This is a title");
            e.author(|a| { 
                a.name("Ferris the SAVAGE");
                a.icon_url("https://rustacean.net/assets/rustacean-flat-happy.png")
            });
            e.colour(Colour::BLUE);
            e.description("No real description to put here");
            e.image("attachment://boom.gif");
            e.fields(vec![
                (":100: Best Stuff", "This is a field body", true),
                ("second field", "Both of these fields are inline", true),
            ]);
            e.field("This is the third field", "This is not an inline field", false);
            e.footer(|f| {
                f.text("This is a footer");
                f.icon_url("https://cdn4.iconfinder.com/data/icons/hospital-19/512/15_hospital-512.png");
                f
            });

            e
        });
        m.add_file(AttachmentType::Path(Path::new("D:/Code/rust/m-bot/resources/gif/boom.gif")));
        m
    });

    
    if let Err(why) = mm {
        error!("Error sending message: {:?}", why);
    }

    Ok(())
}
