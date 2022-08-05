#![warn(clippy::str_to_string)]
mod event_listener;

use poise::serenity_prelude as serenity;
use std::{collections::HashMap, env::var};

use event_listener::event_listener;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
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
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Register application commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::builtins::register_application_commands(ctx, global).await?;

    Ok(())
}

#[poise::command(
    context_menu_command = "View raw Markdown",
    ephemeral,
)]
async fn raw_markdown(ctx: Context<'_>, msg: serenity::Message) -> Result<(), Error> {
    let content = msg.content;

    // escape all backticks 
    let content = content.replace("`", "\\`");

    // escape all markdown formatting
    let content = content.replace("*", "\\*");
    let content = content.replace("_", "\\_");
    let content = content.replace("~", "\\~");
    let content = content.replace(">", "\\>");
    let content = content.replace("|", "\\|");



    // return if more than 2000 characters
    if content.len() >= 3999 {
        ctx.say(
            "Since this message is over 4000 characters, it cannot be displayed in a non embed message. Sorry :( ",
        ).await?;
        return Ok(());
    }

    ctx.send(|msg| {
        msg.embed(|embed| {
            embed.title("Raw Markdown");
            embed.description(format!(
                "\n{}\n",
                content
            ));
            embed
        })
    }).await?;

    Ok(())
}

/// Command for mobile users
#[poise::command(
    slash_command,
    ephemeral,
    rename = "copy-markdown",
    guild_only
)]
async fn copy_markdown(
    ctx: Context<'_>,
    #[description = "The message link"]
    message_link: String,
) -> Result<(), Error> {

    let verify_link_regex = regex::Regex::new(r"https://(?:canary.|ptb.)?discord.com/channels/(?P<serverid>\d+)/(?P<channelid>\d+)/(?P<messageid>\d+)");

    let verify_link = match verify_link_regex {
        Ok(regex) => regex,
        Err(_) => {
            ctx.say("Invalid link").await?;
            return Ok(());
        }
    };
    

    let verify_link_captures = verify_link.captures(&message_link);
    
    let captures = match verify_link_captures {
        Some(captures) => captures,
        None => {
            ctx.say("Invalid link").await?;
            return Ok(());
        }
    };

    let server_id =  captures.name("serverid").unwrap().as_str();
    let channel_id = captures.name("channelid").unwrap().as_str();
    let message_id = captures.name("messageid").unwrap().as_str();

    // if there is a serverid mismatch, return an error
    if server_id != ctx.guild_id().unwrap().as_u64().to_string() {
        ctx.say("The server ID in the message link does not match the server ID of the current server.").await?;
        return Ok(());
    }

    let message = ctx.discord().http.get_message(channel_id.parse::<u64>().unwrap(), message_id.parse::<u64>().unwrap()).await?;

    let content = message.content;

    let content = content.replace("*", "\\*");
    let content = content.replace("_", "\\_");
    let content = content.replace("~", "\\~");
    let content = content.replace(">", "\\>");
    let content = content.replace("|", "\\|");




    ctx.send(|msg| {
        msg.embed(|embed| {
            embed.title("Raw Markdown");
            embed.description(format!(
                "\n{}\n",
                content
            ));
            embed
        })
    }).await?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let framework = poise::Framework::builder()
        .token(var("TOKEN").expect("Missing `TOKEN` env var"))
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data { })
            })
        })
        .options(poise::FrameworkOptions {
            commands: vec![
            help(),
            register(),
            raw_markdown(),
            copy_markdown(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            ..Default::default()
        },
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        /// This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        /// The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        listener: |_ctx, event, _framework, _data| {
            Box::pin(event_listener(_ctx, event, _framework, _data))
        },
        ..Default::default()
        })
        .intents(
           serenity::GatewayIntents::GUILDS | serenity::GatewayIntents::GUILD_MESSAGES | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .run()
        .await
        .unwrap();
}