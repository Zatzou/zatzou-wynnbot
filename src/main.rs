mod commands;
mod error;
mod helpers;
mod wynn;

use std::{
    collections::HashSet,
    sync::{atomic::AtomicU8, Arc},
};

use commands::{gather::*, help::*, id::*, map::*, owner::*, ping::*, up::*};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::{group, hook}, StandardFramework},
    http::Http,
    model::{event::ResumedEvent, gateway::Ready, channel::{Message, ReactionType}},
    prelude::*,
};
use tracing::{error, info, warn, Level};

pub const BOT_NAME: &str = "Zatzoubot";
pub const BOT_VERSION: &str = "0.1.0";

/// Quality level used for .webp exports set from config
pub static WEBP_QUALITY: AtomicU8 = AtomicU8::new(80);

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
#[commands(ping, quit, map, id, maxid, gather, up, sp, help)]
struct General;

#[tokio::main]
async fn main() {
    // config
    let mut config = config::Config::new();
    config
        .merge(config::File::with_name("./config.toml"))
        .expect("Config file 'config.toml' not found in the current directory");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();

    if let Ok(q) = config.get::<u8>("image.webp_quality") {
        WEBP_QUALITY.store(q, std::sync::atomic::Ordering::Relaxed);
    } else {
        warn!("option image.webp_quality not set in config.toml! Using default of 80 instead for .webp export quality")
    }

    let token = config
        .get::<String>("bot.token")
        .expect("Bot token not found in the config file");
    let app_id = config
        .get::<u64>("bot.app_id")
        .expect("App id not found in the config file");

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
        .before(before_hook)
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

#[hook]
async fn before_hook(ctx: &Context, msg: &Message, _: &str) -> bool {
    if msg.author.id == 336039947578376192 {
        //msg.reply(&ctx.http, "<:coinflop:913770918214529084>").await.unwrap();
        msg.react(&ctx.http, ReactionType::Custom { animated: false, id: 913770918214529084.into(), name: Some(String::from("coinflop")) }).await.unwrap();
        return false;
    }
    
    true
}
