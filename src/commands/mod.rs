pub mod voice;

use serenity::client::bridge::gateway::ShardId;
use serenity::model::user::User;
use crate::Context;

#[poise::command(slash_command)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<User>,
) -> Result<(), serenity::Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await.map(|_|{})
}

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