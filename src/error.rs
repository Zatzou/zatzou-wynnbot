use poise::serenity::utils::Color;

use tracing::error;

use crate::{Context, Data, Error};
use crate::{BOT_NAME, BOT_VERSION};

/// Color used for errors
pub const ERROR_COLOR: Color = Color::RED;

/// Function for handling errors
pub async fn error_handler(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx } => {
            if let Some(err) = error.downcast_ref::<reqwest::Error>() {
                create_error_msg(
                    ctx,
                    "Failed to load an external resource",
                    &format!("{}", err),
                )
                .await;
            } else {
                create_error_msg(
                    ctx,
                    "An unknown internal error has occured",
                    &format!("{:?}", error),
                )
                .await;
            }
        }
        _ => {
            error!("Unhandled error occured");
        }
    }
}

/// Function for sending error messages easily
pub async fn create_error_msg(ctx: Context<'_>, title: &str, desc: &str) {
    let errormsg = ctx
        .send(|m| {
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
        .await;

    if let Err(why) = errormsg {
        error!("Failed to send an error message for error: `{} {}` because another error occured while sending the error message: {}", title, desc, why);
    }
}
