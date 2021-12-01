use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::helpers::parse_command_args_raw;

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let command = parse_command_args_raw(msg);

    if let Some(cmd) = command {
        match cmd.to_lowercase().as_str() {
            "help" => send_help_msg(".help", "gets help messages\nUsage:\n.help (command)", ctx, msg).await?,
            "map" => send_help_msg(".map", "renders the guild map\nUsage:\n.map", ctx, msg).await?,
            "gather" => send_help_msg(".gather", "Finds gather spots and renders them to a map\nUsage:\n.gather [material]", ctx, msg).await?,
            "up" => send_help_msg(".up", "Gets the uptimes of the current servers or more specific info on a single server\nUsage:\n.up (server number)", ctx, msg).await?,
            "sp" => send_help_msg(".sp", "Gets the approximate sp regen times for all servers", ctx, msg).await?,
            "id" => send_help_msg(".id", "Reads wynntils id strings\nUsage:\n.id [id string]", ctx, msg).await?,
            "maxid" => send_help_msg(".maxid", "Returns a perfect id string for the given item\nUsage:\n.maxid [item name (case sensitive)]", ctx, msg).await?,
            _ => send_help_msg("Unrecognized command", &format!("The command `{}` doesn't seem to exist or have any help", cmd), ctx, msg).await?,
        }
    } else {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Zatzoubot commands");

                e.field(".map","Renders the current guild map", false);
                e.field(".gather", "Searches for gatherable materials\nUsage:\n.gather [material]", false);
                e.field(".up", "Gets the server uptimes for wynn servers or a specific server\nUsage:\n.up (server number)", false);
                e.field(".sp", "Gets the approximate sp regen times for all servers", false);
                e.field(".id", "Reads wynntils id strings\nUsage:\n.id [id string]", false);
                e.field(".maxid", "Creates an 100% wynntils id string for an item\nUsage:\n.id [Item name (case sensitive)]", false);

                e.description("() optional parameter\n[] required parameter\n\nUse .help (command) for more specific help");
                e
            });
            m
        }).await?;
    }

    Ok(())
}

async fn send_help_msg(title: &str, body: &str, ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(title);
                e.description(body);
                e
            });
            m
        })
        .await?;

    Ok(())
}
