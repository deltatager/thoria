use songbird::driver::Bitrate;
use crate::Context;

/// Join your current voice channel
#[poise::command(slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), serenity::Error> {
    let guild = ctx.guild().expect("Could not get guild");
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            return ctx.say("Not in a voice channel").await.map(|_| {});
        }
    };

    let manager = songbird::get(ctx.discord()).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    ctx.say(format!("Joined {}", connect_to.name(ctx.discord().cache.clone()).await.expect(""))).await.map(|_| {})
}

/// Play a track from an URL
#[poise::command(slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URL of track to play"]
    track: Option<String>,
) -> Result<(), serenity::Error> {
    let guild_id = ctx.guild().expect("").id;
    let manager = songbird::get(ctx.discord())
        .await
        .unwrap()
        .clone();

    let url = match track {
        Some(url) => url,
        None => {
            if let Some(handler_lock) = manager.get(guild_id) {
                let handler = handler_lock.lock().await;
                handler.queue().resume().ok();
                return ctx.say("Resumed playing").await.map(|_| {});
            }
                return ctx.say("i shat myself").await.map(|_| {});
        },
    };

    if !url.starts_with("http") {
        return ctx.say("Must provide a valid URL").await.map(|_| {});
    }

    return if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match songbird::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                return ctx.say("Error sourcing ffmpeg").await.map(|_| {});
            }
        };

        handler.set_bitrate(Bitrate::Max);
        let track = handler.enqueue_source(source);
        track.set_volume(0.33).ok();

        ctx.say("Playing song").await.map(|_| {})
    } else {
        ctx.say("Not in a voice channel to play in").await.map(|_| {})
    };
}

#[poise::command(slash_command)]
pub async fn pause(ctx: Context<'_>) -> Result<(), serenity::Error> {
    let guild_id = ctx.guild().expect("").id;

    let manager = songbird::get(ctx.discord())
        .await
        .unwrap()
        .clone();

    return if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        handler.queue().pause().ok();
        ctx.say("Pausing").await.map(|_| {})
    } else {
        ctx.say("Not in a voice channel to play in").await.map(|_| {})
    };
}

#[poise::command(slash_command)]
pub async fn bitrate(
    ctx: Context<'_>,
    bitrate: i32
) -> Result<(), serenity::Error> {
    let guild_id = ctx.guild().expect("").id;

    let manager = songbird::get(ctx.discord())
        .await
        .unwrap()
        .clone();

    return if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        handler.set_bitrate(Bitrate::BitsPerSecond(bitrate));
        ctx.say(format!("Changing bitrate to {} bps", bitrate)).await.map(|_| {})
    } else {
        ctx.say("Not in a voice channel to play in").await.map(|_| {})
    };
}