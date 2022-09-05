use serenity::client::bridge::gateway::ShardId;

use context::Context;

pub mod voice;
pub mod context;

#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), serenity::Error> {
    let latency = ctx
        .framework()
        .shard_manager
        .lock()
        .await
        .runners
        .lock()
        .await
        .get(&ShardId(ctx.discord().shard_id))
        .expect("Shard not found")
        .latency
        .map(|d| format!("{:#?}", d))
        .unwrap_or("Could not get latency".to_string());

    ctx.say(latency).await.map(|_|{})
}