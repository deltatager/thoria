extern crate core;

use serenity::prelude::GatewayIntents;
use songbird::SerenityInit;

use crate::commands::{help, ping, register, shutdown};
use crate::commands::voice::{bitrate, join, pause, play, queue, stop};

mod commands;
mod context;
mod source;
mod opus_source;

// User data, which is stored and accessible in all command invocations
pub struct Data {}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token =
        std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN should be provided in the env");

    poise::Framework::builder()
        .client_settings(|b| {b.register_songbird()})
        .options(poise::FrameworkOptions {
            commands: vec![register(), shutdown(), help(), ping(), join(), play(), pause(), stop(), bitrate(), queue()],
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