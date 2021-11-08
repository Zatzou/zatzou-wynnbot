use std::collections::HashMap;

use serenity::{client::Context, framework::standard::CommandResult, model::channel::Message};
use serenity::framework::standard::macros::command;

use tracing::info;
use serde::Deserialize;
use cached::proc_macro::cached;

use crate::error::create_error_msg;
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

        single_server(ctx, msg, server).await?;
    }
    
    Ok(())
}

async fn server_list(ctx: &Context, msg: &Message) -> CommandResult {
    let servers = get_servers().await?;

    let mut desc = String::new();
    
    desc.push_str("```css\n");

    for server in servers {
        let times = parse_timestamp(server.started);
        if times.0 == 0 {
            desc.push_str(&format!("{:<4} | {:>2} |    {:>2}m\n", server.name, server.players.len(), times.1));    
        } else {
            desc.push_str(&format!("{:<4} | {:>2} | {}h {:>2}m\n", server.name, server.players.len(), times.0, times.1));
        }
    }

    desc.push_str(&format!("```\nData from <t:{}:R>", chrono::offset::Utc::now().timestamp()));

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

async fn single_server(ctx: &Context, msg: &Message, servernum: i32) -> CommandResult {
    let servers = get_servers().await?;

    let server = servers.into_iter().find(|s| s.name.trim() == format!("WC{}", servernum).trim());

    if let Some(server) = server {
        let mut plist = String::new();

        for name in server.players {
            plist.push_str(&[&name, "\n"].concat());
        }

        msg.channel_id
        .send_message(&ctx.http, |m| {
            m.add_embed(|e| {
                e.title(&format!("WC{}", servernum));
                e.description(&format!("The server WC{} started <t:{}:R>\nIt has been running since <t:{}:T>\n\nPlayer list\n```\n{}```", servernum, server.started / 1000, server.started / 1000, plist));
                e
            });
            m
        }).await?;
    } else {
        create_error_msg(ctx, msg, "Server not found", "The given server is either not online or it is newer than 5 minutes").await;
    }

    Ok(())
}

fn parse_timestamp(timestamp: i64) -> (i64, i64) {
    let now = chrono::offset::Utc::now().timestamp();
    // divide the original timestamp by 1000 to get the actual time since wynntils uses milliseconds
    let timestamp = timestamp / 1000;

    let uptime = now - timestamp;

    let hours = uptime / 3600;
    let minutes = (uptime % 3600) / 60;

    (hours, minutes)
}
