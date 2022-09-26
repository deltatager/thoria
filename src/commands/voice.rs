use hhmmss::Hhmmss;
use serenity::futures::future::join_all;
use songbird::driver::Bitrate;

use crate::context::{Context, GetManagerTrait, UserKey};
use crate::source::ytdl_native;

/// Join your current voice channel
#[poise::command(slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), serenity::Error> {
    ctx.defer().await.ok();
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

    let manager = songbird::get(ctx.discord()).await.unwrap().clone();
    let _handler = manager.join(guild_id, connect_to).await;
    let msg = format!("Joined {}", connect_to.name(ctx.discord().cache.clone()).await.unwrap());
    ctx.say(msg).await.map(|_| {})
}

/// Play a track from an URL
#[poise::command(slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URL of track to play"]
    track: Option<String>,
) -> Result<(), serenity::Error> {
    ctx.defer().await.ok();
    let handler_lock = match ctx.get_handler().await {
        Some(arc) => arc,
        None => {
            return ctx.say("Not in a voice channel").await.map(|_| {});
        }
    };
    let mut handler = handler_lock.lock().await;

    let url = match track {
        Some(url) => if url.starts_with("http") {
            url
        } else {
            let mut string = "ytsearch1:".to_owned();
            string.push_str(url.as_str());
            string
        },
        None => {
            handler.queue().resume().ok();
            return ctx.say("Resumed playing").await.map(|_| {});
        }
    };

    let source = match ytdl_native(&url).await {
        Ok(source) => source,
        Err(why) => {
            return ctx.say(format!("Err getting source: {:?}", why)).await.map(|_| {});
        }
    };

    let track = handler.enqueue_source(source);
    track.set_volume(0.33).ok();
    track.typemap().write().await.insert::<UserKey>(ctx.author().clone());

    ctx.say(format!("Playing `{}`", track.metadata().title.clone().unwrap())).await.map(|_| {})
}

#[poise::command(slash_command)]
pub async fn pause(ctx: Context<'_>) -> Result<(), serenity::Error> {
    ctx.defer().await.ok();
    let handler = match ctx.get_handler().await {
        Some(arc) => arc,
        None => {
            return ctx.say("Not in a voice channel").await.map(|_| {});
        }
    };

    handler.lock().await.queue().pause().ok();
    ctx.say("Pausing").await.map(|_| {})
}

#[poise::command(slash_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), serenity::Error> {
    ctx.defer().await.ok();
    let handler = match ctx.get_handler().await {
        Some(arc) => arc,
        None => {
            return ctx.say("Not in a voice channel").await.map(|_| {});
        }
    };

    handler.lock().await.queue().stop();
    ctx.say("Stopped.").await.map(|_| {})
}

#[poise::command(slash_command, owners_only)]
pub async fn bitrate(
    ctx: Context<'_>,
    bitrate: i32,
) -> Result<(), serenity::Error> {
    ctx.defer().await.ok();
    let handler = match ctx.get_handler().await {
        Some(arc) => arc,
        None => {
            return ctx.say("Not in a voice channel").await.map(|_| {});
        }
    };

    handler.lock().await.set_bitrate(Bitrate::BitsPerSecond(bitrate));
    ctx.say(format!("Changing bitrate to {} bps", bitrate)).await.map(|_| {})
}

/// Prints the current track queue
#[poise::command(slash_command)]
pub async fn queue(ctx: Context<'_>) -> Result<(), serenity::Error> {
    ctx.defer().await.ok();
    let handler = match ctx.get_handler().await {
        Some(arc) => arc,
        None => {
            return ctx.say("Not in a voice channel").await.map(|_| {});
        }
    };

    let queue = handler.lock().await.queue().current_queue();

    if queue.len() == 0 {
        return ctx.say("Queue is empty.").await.map(|_| {});
    }

    let lines: Vec<_> = queue.iter()
        .map(|th| async {
            format!(
                "{}. [{:#?}] - {} - {}\n",
                0,
                th.metadata().duration.clone().unwrap().hhmmss(),
                th.metadata().title.clone().unwrap(),
                th.typemap().read().await.get::<UserKey>().unwrap()
            )
        })
        .collect();

    let msg: String = join_all(lines).await.into_iter().collect();
    ctx.say(msg).await.map(|_| {})
}