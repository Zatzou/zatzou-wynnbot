mod commands;
mod config;
mod error;
mod help;
mod wynn;

use commands::{gather, id, map, up};
use config::Config;
use poise::serenity_prelude::{self as serenity, ComponentType, Interaction};

use tracing::{error, info, log::warn, Level};

pub const BOT_NAME: &str = "Zatzoubot";
pub const BOT_VERSION: &str = "0.1.0";

pub struct Data {
    config: Config,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// handle discord events
async fn event_listener(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: &poise::Framework<Data, Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            info!("{} is connected!", data_about_bot.user.name)
        }
        poise::Event::InteractionCreate { interaction } => match interaction {
            Interaction::MessageComponent(intr) => {
                if intr.data.component_type == ComponentType::Button {
                    let msg = &intr.message;

                    let result = match intr.data.custom_id.as_ref() {
                        "update_sp" => {
                            crate::commands::up::sp_interact_handler(ctx, &msg, &intr).await
                        }
                        _ => {
                            warn!("Button with id `{}` pressed but there is no handler for a button with that id", intr.data.custom_id);
                            Ok(())
                        }
                    };

                    if let Err(why) = result {
                        error!("Interaction failed: {}", why);
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::builtins::register_application_commands(ctx, global).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    // initialize the logger with a default log level
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Read the config file
    let config = config::read_config();

    // poise options
    let options = poise::FrameworkOptions {
        commands: vec![
            map::map(),
            up::up(),
            up::sp(),
            id::id(),
            id::maxid(),
            gather::gather(),
            help::help(),
        ],
        listener: |ctx, event, framework, user_data| {
            Box::pin(event_listener(ctx, event, framework, user_data))
        },
        on_error: |error| Box::pin(crate::error::error_handler(error)),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: if config.commands.enable_prefix_cmds {
                Some(config.commands.cmd_prefix.clone())
            } else {
                None
            },
            mention_as_prefix: false,
            edit_tracker: None,
            ..Default::default()
        },
        ..Default::default()
    };

    let bot = poise::Framework::build()
        .token(config.bot.get_token())
        .options(options)
        .user_data_setup(|ctx, _bot_data, framework| {
            Box::pin(async move {
                // register the application commands
                info!("Registering application commands globally");
                let mut cmd_builder = serenity::CreateApplicationCommands::default();
                let cmds = &framework.options().commands;

                for cmd in cmds {
                    if let Some(slash_cmd) = cmd.create_as_slash_command() {
                        cmd_builder.add_application_command(slash_cmd);
                    }
                    if let Some(ctxmenu_cmd) = cmd.create_as_context_menu_command() {
                        cmd_builder.add_application_command(ctxmenu_cmd);
                    }
                }

                let cmd_builder = serenity::json::Value::Array(cmd_builder.0);

                let create = ctx
                    .http
                    .create_global_application_commands(&cmd_builder)
                    .await;

                if let Err(why) = create {
                    error!("Failed to register app commands: {}", why);
                } else {
                    info!("Application commands registered successfully");
                }

                let shard_manager = framework.shard_manager().clone();

                tokio::spawn(async move {
                    tokio::signal::ctrl_c()
                        .await
                        .expect("Could not register ctrl+c handler");
                    shard_manager.lock().await.shutdown_all().await;
                });

                // Initialize the data struct
                Ok(Data { config })
            })
        });

    if let Err(why) = bot.run().await {
        error!("Bot failed to start: {:?}", why);
    }
}
