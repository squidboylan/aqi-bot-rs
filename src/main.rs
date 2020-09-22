use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

mod purple_air;

struct Handler;

async fn send_msg(ctx: Context, msg: Message, data: &str) {
    if let Err(why) = msg.channel_id.say(&ctx.http, data).await {
        println!("Error sending message: {:?}", why);
    }
}

fn create_msg(data: purple_air::Response) -> String {
    format!(
        "id: {}, pm2.5 data: current {:.1}; 10 min {:.1}; 30 min {:.1}; 6 hour {:.1}; 24 hour {:.1}",
        data.results[0].id,
        purple_air::raw_to_aqi(data.results[0].stats.v),
        purple_air::raw_to_aqi(data.results[0].stats.v1),
        purple_air::raw_to_aqi(data.results[0].stats.v2),
        purple_air::raw_to_aqi(data.results[0].stats.v4),
        purple_air::raw_to_aqi(data.results[0].stats.v5)
    )
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        println!("{}", msg.content);
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            send_msg(ctx, msg, "Pong!").await;
        } else if msg.content.starts_with("!aqi ") {
            let msg_list: Vec<&str> = msg.content.split_whitespace().collect();
            if msg_list.len() < 2 {
                send_msg(ctx, msg, "!aqi requires the sensor ID as an argument").await;
                return;
            }
            let id = msg_list[1].parse::<u64>();
            if let Err(_) = id {
                send_msg(ctx, msg, "Failed to parse id").await;
                return;
            }
            let id = id.unwrap();
            let res = purple_air::get_sensor_data(id).await;
            if let Err(why) = res {
                send_msg(ctx, msg, &why.to_string()).await;
                return;
            }
            let json = res.unwrap();
            send_msg(ctx, msg, &create_msg(json)).await;
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
