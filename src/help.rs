use std::collections::HashSet;

use serenity::{client::Context, model::{channel::Message, id::UserId}, framework::standard::{Args, HelpOptions, CommandGroup, macros::help, CommandResult, help_commands}};

#[help]
#[embed_error_colour("#E74C3C")]
#[command_not_found_text("Command not found")]
async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    
    Ok(())
}