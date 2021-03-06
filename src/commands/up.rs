use poise::serenity::model::interactions::message_component::{
    ButtonStyle, MessageComponentInteraction,
};
use poise::serenity::model::interactions::InteractionResponseType;
use poise::serenity_prelude::Message;

use crate::{Context, Error};

use crate::error::create_error_msg;

use crate::wynn::Servers::*;

/// Check uptimes of servers
#[poise::command(prefix_command, slash_command)]
pub async fn up(
    ctx: Context<'_>,
    #[description = "Server number"] id: Option<i32>,
) -> Result<(), Error> {
    if let Some(n) = id {
        single_server(ctx, n).await?;
    } else {
        server_list(ctx).await?;
    }

    Ok(())
}

async fn server_list(ctx: Context<'_>) -> Result<(), Error> {
    let mut servers = get_servers().await?;
    // sort servers by uptime
    servers.sort_unstable_by_key(|s| s.started);
    servers.reverse();

    let mut desc = String::new();

    desc.push_str("```css\n");

    for server in servers {
        let times = parse_timestamp(server.started);
        if times.0 == 0 {
            desc.push_str(&format!(
                "{:>4} | {:>2} |      {:>2}m\n",
                server.name,
                server.players.len(),
                times.1
            ));
        } else {
            desc.push_str(&format!(
                "{:>4} | {:>2} |  {}h  {:>2}m\n",
                server.name,
                server.players.len(),
                times.0,
                times.1
            ));
        }
    }

    desc.push_str("```");

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Server | Players | Uptime");
            e.description(desc);
            e.timestamp(chrono::Utc::now().to_rfc3339());
            e
        });
        m
    })
    .await?;

    Ok(())
}

async fn single_server(ctx: Context<'_>, servernum: i32) -> Result<(), Error> {
    let servers = get_servers().await?;

    let server = servers
        .into_iter()
        .find(|s| s.name.trim() == format!("WC{}", servernum).trim());

    if let Some(server) = server {
        let mut plist = String::new();

        for name in server.players {
            plist.push_str(&[&name, "\n"].concat());
        }

        let times = parse_timestamp(server.started);

        ctx
        .send(|m| {
            m.embed(|e| {
                e.title(&format!("WC{}", servernum));
                e.description(&format!("The server WC{} started <t:{}:T>\nIt has been running for `{}h {:>2}m {:>2}s`\n\nPlayer list\n```\n{}```", servernum, server.started / 1000, times.0, times.1, times.2, plist));
                e.timestamp(chrono::Utc::now().to_rfc3339());
                e
            });
            m
        }).await?;
    } else {
        create_error_msg(
            ctx,
            "Server not found",
            "The given server is either not online or it is newer than 5 minutes",
        )
        .await;
    }

    Ok(())
}

fn parse_timestamp(timestamp: i64) -> (i64, i64, i64) {
    let now = chrono::offset::Utc::now().timestamp();
    // divide the original timestamp by 1000 to get the actual time since wynntils uses milliseconds
    let timestamp = timestamp / 1000;

    let uptime = now - timestamp;

    let hours = uptime / 3600;
    let minutes = (uptime % 3600) / 60;
    let seconds = uptime % 60;

    (hours, minutes, seconds)
}

/// Get sp regen times
#[poise::command(prefix_command, slash_command)]
pub async fn sp(ctx: Context<'_>) -> Result<(), Error> {
    let desc = generate_sp_table().await?;

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Soulpoint regen times (offset 2m)");
            e.description(desc);
            e.timestamp(chrono::Utc::now().to_rfc3339());
            e
        });
        m.components(|c| {
            c.create_action_row(|ar| {
                ar.create_button(|b| {
                    b.style(ButtonStyle::Primary);
                    b.label("Update");
                    b.custom_id("update_sp");
                    b.disabled(false);
                    b
                });
                ar
            });
            c
        });
        m
    })
    .await?;

    Ok(())
}

async fn generate_sp_table() -> Result<String, reqwest::Error> {
    let mut servers = get_servers().await?;
    let now = chrono::offset::Utc::now().timestamp();

    for s in servers.iter_mut() {
        s.started = 1200 - ((now - (s.started + 120) / 1000) % 1200);
    }

    servers.sort_unstable_by_key(|s| s.started);

    let mut desc = String::new();

    desc.push_str("Server | Players | Sp regen\n```css\n");

    for server in servers.iter().take(25) {
        let minutes = server.started / 60;
        let seconds = server.started % 60;
        if minutes == 0 {
            desc.push_str(&format!(
                "{:>4} | {:>2} |      {:>2}s\n",
                server.name,
                server.players.len(),
                seconds
            ));
        } else {
            desc.push_str(&format!(
                "{:>4} | {:>2} | {:>2}m  {:>2}s\n",
                server.name,
                server.players.len(),
                minutes,
                seconds
            ));
        }
    }

    desc.push_str("```\nNote:\nsp regen times are approximate");

    Ok(desc)
}

async fn update_sp_msg(
    ctx: &poise::serenity_prelude::Context,
    msg: &mut Message,
    updatebtn: bool,
) -> Result<(), Error> {
    let desc = generate_sp_table().await?;

    msg.edit(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Soulpoint regen times (offset 2m)");
            e.description(desc);
            e.timestamp(chrono::Utc::now().to_rfc3339());
            e
        });
        // only add the updatebtn when we want it
        if updatebtn {
            m.components(|c| {
                c.create_action_row(|ar| {
                    ar.create_button(|b| {
                        b.style(ButtonStyle::Primary);
                        b.label("Update");
                        b.custom_id("update_sp");
                        b.disabled(false);
                        b
                    });
                    ar
                });
                c
            });
        }
        m
    })
    .await?;

    Ok(())
}

pub async fn sp_interact_handler(
    ctx: &poise::serenity_prelude::Context,
    msg: &Message,
    interact: &MessageComponentInteraction,
) -> Result<(), Error> {
    let mut message = msg.clone();

    update_sp_msg(ctx, &mut message, true).await?;

    interact
        .create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::DeferredUpdateMessage);
            r
        })
        .await?;

    Ok(())
}
