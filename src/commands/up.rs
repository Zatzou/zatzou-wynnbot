use std::collections::HashMap;

use serenity::{client::Context, framework::standard::CommandResult, model::channel::Message};
use serenity::framework::standard::macros::command;

use tracing::info;
use serde::Deserialize;
use cached::proc_macro::cached;

use crate::helpers::parse_command_args;

#[cached(time = 300, result = true)]
async fn get_servers() -> Result<Vec<ParsedServer>, reqwest::Error> {
    info!("Getting new server data from wynntils");
    let servers: ServerList = reqwest::get("https://athena.wynntils.com/cache/get/serverList")
        .await?
        .json()
        .await?;
    let mut parsed = Vec::new();
    for (k, v) in servers.servers.into_iter() {
        parsed.push(ParsedServer {
            name: k,
            started: v.firstSeen,
            players: v.players,
        })
    }
    parsed.sort_unstable_by_key(|s| s.started);
    parsed.reverse();
    Ok(parsed)
}

#[derive(Clone, Deserialize)]
struct ServerList {
    servers: HashMap<String, Server>,
}

#[derive(Clone, Deserialize)]
#[allow(non_snake_case)]
struct Server {
    firstSeen: i64,
    players: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ParsedServer {
    name: String,
    started: i64,
    players: Vec<String>
}

#[command]
async fn up(ctx: &Context, msg: &Message) -> CommandResult {
    let cmd_args = parse_command_args(msg);
    
    if cmd_args.len() == 1 {
        server_list(ctx, msg).await?;
    } else {
        let server: i32 = cmd_args.get(1).unwrap().parse()?;
    }
    
    Ok(())
}

async fn server_list(ctx: &Context, msg: &Message) -> CommandResult {
    let servers = get_servers().await?;

    let mut desc = String::new();
    
    for server in servers {
        desc.push_str(&format!("`{:<4} | {:>2} |` <t:{}:R>\n", server.name, server.players.len(), server.started / 1000));
    }

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.add_embed(|e| {
                e.title("Server | Players | Uptime");
                e.description(desc);
                e
            });
            m
        }).await?;

    Ok(())
}
