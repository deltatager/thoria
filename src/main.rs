mod commands;

use serenity::Error;
use serenity::prelude::GatewayIntents;
use crate::commands::{age, ping};

type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {}

#[poise::command(prefix_command, owners_only, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await
}

#[poise::command(slash_command, owners_only, hide_in_help)]
async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.framework()
        .shard_manager()
        .lock()
        .await
        .shutdown_all()
        .await;
    Ok(())
}

#[poise::command(slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "\
This is an example bot made to showcase features of my custom Discord bot framework",
            show_context_menu_commands: true,
            ephemeral: false,
            ..Default::default()
        },
    )
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token =
        std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN should be provided in the env");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![register(), shutdown(), help(), age(), ping()],
            ..Default::default()
        })
        .token(token)
        .intents(GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                let shard_manager = _framework.shard_manager().clone();
                tokio::spawn(async move {
                    tokio::signal::ctrl_c()
                        .await
                        .expect("Could not register ctrl+c handler");
                    println!("Shutting down...");
                    shard_manager.lock().await.shutdown_all().await;
                });

                Ok(Data {})
            })
        });

    framework.build().await.expect("").start_with(async |client| {
        client.start()
    }).await.expect("")
}
