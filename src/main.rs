mod commands;
mod error;
mod wynn;
mod helpers;

use std::{collections::HashSet, sync::Arc};

use commands::{map::*, owner::*, ping::*, id::*};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use tracing::{error, info, Level};

pub const BOT_NAME: &str = "Zatzoubot";
pub const BOT_VERSION: &str = "0.1.0";

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(ping, quit, map, id, maxid, gather)]
struct General;

#[tokio::main]
async fn main() {
    // config
    let mut config = config::Config::new();
    config
        .merge(config::File::with_name("./config.toml"))
        .unwrap();

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();

    let token = config.get::<String>("bot.token").unwrap();
    let app_id = config.get::<u64>("bot.app_id").unwrap();

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("."))
        .bucket("map", |b| b.delay(5).time_span(60).limit(5))
        .await
        .group(&GENERAL_GROUP)
        .after(error::command_error_hook);

    let mut client = Client::builder(&token)
        .application_id(app_id)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        // shardmanager
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
