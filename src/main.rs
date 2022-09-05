use serenity::Error;
use serenity::prelude::GatewayIntents;
use songbird::SerenityInit;

use commands::context::Context;

use crate::commands::ping;
use crate::commands::voice::{bitrate, join, pause, play, queue};

mod commands;

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
            extra_text_at_bottom: "\nThis is an example bot made to showcase features of my custom Discord bot framework",
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

    poise::Framework::builder()
        .client_settings(|b| {b.register_songbird()})
        .options(poise::FrameworkOptions {
            commands: vec![register(), shutdown(), help(), ping(), join(), play(), pause(), bitrate(), queue()],
            ..Default::default()
        })
        .token(token)
        .intents(GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| { Box::pin(async move { Ok(Data {}) }) })
        .build()
        .await
        .expect("Error building framework")
        .start_with(|mut client| async move {
            let shard_manager = client.shard_manager.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Could not register ctrl+c handler");
                println!("Shutting down...");
                shard_manager.lock().await.shutdown_all().await;
            });

            client.start().await
        })
        .await
        .expect("Error starting client")
}
