#[poise::command(slash_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), serenity::Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await.map(|_|{})
}

#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), serenity::Error> {
    let latency = ctx
        .framework()
        .shard_manager
        .lock()
        .await
        .runners
        .lock()
        .await
        .get(&serenity::ShardId(ctx.discord().shard_id))
        .expect("Shard not found")
        .latency
        .map(|d| format!("{:#?}", d))
        .unwrap_or("Could not get latency".to_string());

    ctx.say(latency).await.map(|_|{})
}