use crate::{Context, Error};

/// Help command
#[poise::command(prefix_command, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help of"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;

    Ok(())
}
