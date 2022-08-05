
type Error = Box<dyn std::error::Error + Send + Sync>;
use poise::serenity_prelude as serenity;
use ::serenity::gateway::ConnectionStage;
use crate::Data;


pub async fn event_listener(
    _ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!("[0;34m[INFO][0m {} is connected!", data_about_bot.user.name);
            println!("[0;34m[INFO][0m Using API v{}", data_about_bot.version);
            println!("[0;34m[INFO][0m Session Id: {}", data_about_bot.session_id);
            // set status
            /* let activity   = serenity_rs::Activity::playing("Creating some more bugs :^)");
            let status =  serenity_rs::OnlineStatus::DoNotDisturb;
            ctx.set_presence(Some(activity), status).await; */
        }
        // [0;31;1;4;5m test [0m
        poise::Event::ShardStageUpdate { update } => {
            let info = resolve_shard_stages(update.new, update.shard_id).await;
            println!("{}", info)
        }
        poise::Event::GuildCreate { guild, is_new } => {
            println!(
                "[0;33m[GUILDS][0m Guild[0;31m {}[0m became [0;32mavailable[0m",
                guild.name
            );
            if *is_new {
                println!("[0;33m[GUILDS][0m Joined [0;31m {}[0m", guild.name);
                // create db entry for guild
            }
        }
        poise::Event::GuildDelete { incomplete, full } => {
            let guild = full.clone().unwrap();
            if !incomplete.unavailable {
                println!(
                    "[0;33m[GUILDS][0m Got banned/kicked from guild [0;31m {}[0m ",
                    &guild.name
                );
            } else {
                println!("[0;33m[GUILDS][0m Guild [0;31m {}[0m became unavailable, most likely due to a server outage",
                    &guild.name
                );
            }
        }

        _ => {}
    }

    Ok(())
}

async fn resolve_shard_stages(stage: ConnectionStage, shard_id: serenity::ShardId) -> String { 
    let stage = match stage {
    ConnectionStage::Connected => "🟢 [0;32mConnected[0m",
    ConnectionStage::Connecting => "🟡 [0;32mConnecting[0m",
    ConnectionStage::Disconnected => "🔥 [0;31mDisconnected[0m",
    ConnectionStage::Handshake => "🤝 [0;32mhandshaking[0m",
    ConnectionStage::Identifying => "🔎 [0;32mIdentifying[0m",
    ConnectionStage::Resuming => "🟡 [0;32mResuming[0m",
        _ => "unknown",
    };
    format!("[0;33m[SHARD][0m Shard {} is in stage {}", shard_id, stage)
}