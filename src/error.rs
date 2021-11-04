use serenity::client::Context;
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;
use serenity::utils::Color;

use crate::{BOT_NAME, BOT_VERSION};

const ERROR_COLOR: Color = Color::RED;

#[hook]
pub async fn command_error_hook(
    ctx: &Context,
    msg: &Message,
    _cmd_name: &str,
    error: Result<(), CommandError>,
) {
    if let Err(error) = error {
        if let Some(err) = error.downcast_ref::<reqwest::Error>() {
            create_error_msg(
                ctx,
                msg,
                "Failed to load an external resource",
                &format!("{}", err),
            )
            .await;
        } else {
            create_error_msg(
                ctx,
                msg,
                "An unknown internal error has occured",
                &format!("{}", error),
            )
            .await;
        }
    }
}

pub async fn create_error_msg(ctx: &Context, msg: &Message, title: &str, desc: &str) {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(ERROR_COLOR);
                e.title(title);
                e.description(desc);
                e.footer(|f| {
                    f.text(format!("{} {}", BOT_NAME, BOT_VERSION));
                    f
                });
                e
            });
            m
        })
        .await
        .unwrap();
}
