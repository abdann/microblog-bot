#![warn(clippy::str_to_string)]

mod commands;
mod json_models;

use dotenvy::dotenv;
use json_models::Login;
use log::{error, info};
use poise::serenity_prelude as serenity;
use std::{env::var, time::Duration};
use tokio::select;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(windows)]
use tokio::signal::windows::{signal, SignalKind};
use tokio::sync::Mutex;
// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub client: reqwest::Client,
    pub logged_in: Mutex<bool>,
    pub credentials: Login,
    pub csrf_token: Mutex<Option<String>>,
    pub login_endpoint: String,
    pub post_endpoint: String,
    pub domain: String,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            error!(target: "error-logger", "Error in command `{}`: {:?}.", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!(target: "error-logger", "Error while handling error: {}.", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().expect("No .env file found");
    let USERNAME = var("BLOG_USERNAME").expect("No username found");
    let PASSWORD = var("BLOG_PASSWORD").expect("No password found");
    let POST_ENDPOINT = var("POST_ENDPOINT").expect("No 'POST_ENDPOINT' found");
    let LOGIN_ENDPOINT = var("LOGIN_ENDPOINT").expect("No 'LOGIN_ENDPOINT' found");
    let HOSTNAME = var("BLOG_HOSTNAME").expect("No 'BLOG_HOSTNAME' found");
    let SCHEME = var("SCHEME").expect("No 'SCHEME' found");

    let login_info = Login {
        username: USERNAME,
        password: PASSWORD,
    };
    log4rs::init_file("log4rs-config.yml", Default::default())
        .expect("Failed to initialize log4rs from config file.");
    let options = poise::FrameworkOptions {
        commands: vec![commands::help(), commands::microblog()],
        prefix_options: poise::PrefixFrameworkOptions {
            // Prefix is !
            prefix: Some("microblog".into()),
            // Track edits in command messages for the next hour
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        skip_checks_for_owners: true,
        ..Default::default()
    };

    let framework_builder = poise::Framework::builder()
        .token(var("DISCORD_TOKEN").expect("Missing `DISCORD_TOKEN` env var."))
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    client: reqwest::ClientBuilder::new()
                        .https_only(true)
                        .cookie_store(true)
                        .build()
                        .expect("Unable to build client"),
                    logged_in: Mutex::new(false),
                    credentials: login_info,
                    csrf_token: Mutex::new(None),
                    login_endpoint: format!("{}://{}{}", &SCHEME, &HOSTNAME, &LOGIN_ENDPOINT),
                    post_endpoint: format!("{}://{}{}", &SCHEME, &HOSTNAME, &POST_ENDPOINT),
                    domain: format!("{}://{}", &SCHEME, &HOSTNAME),
                })
            })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::GUILDS
                | serenity::GatewayIntents::GUILD_MESSAGES
                | serenity::GatewayIntents::DIRECT_MESSAGES
                | serenity::GatewayIntents::MESSAGE_CONTENT,
        );
    // Build framework
    let framework: std::sync::Arc<
        poise::Framework<Data, Box<dyn std::error::Error + Send + Sync>>,
    > = framework_builder
        .build()
        .await
        .expect("Failed to build framework.");
    // Make SIGTERM and SIGINT listener
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to make SIGTERM listener.");
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to make SIGINT listener.");
    // Make futures to start bot and listen for SIGTERM
    // Join the threads, returning when any one thread completes.
    select! {
    _ = async {
        info!(target: "startup-shutdown-logger", "Bot is starting.");
        framework.client().start().await.unwrap();
    } => {}
    _ =  async {
        // When SIGTERM is received, we teardown the bot.
        sigterm.recv().await;
        teardown(framework.shard_manager(), "Received SIGTERM.").await;
    } => {}
    _ = async {
        // Also teardown the bot if SIGINT is received.
        sigint.recv().await;
        println!("Shutting down gracefully.");
        teardown(framework.shard_manager(), "Received SIGINT.").await;
    } => {}};
}

/// Tears down the program. Should be called when `main` receives `SIGTERM`
async fn teardown(
    shard_manager: &std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>>,
    reason: &str,
) {
    // Shutdown bot
    info!(target: "startup-shutdown-logger", "Shutting down bot. Reason: {}", reason);
    shard_manager.lock().await.shutdown_all().await;
    // Log shutdown
    info!(target: "startup-shutdown-logger", "Bot is shutdown.");
}
