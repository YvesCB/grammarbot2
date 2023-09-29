use db_interactions as dbi;
use events::my_event_handler;
use log::{error, info, warn};
use poise::serenity_prelude as serenity;
use types::*;

mod commands_util;
mod db_interactions;
mod events;
mod logging;
mod types;

#[tokio::main]
async fn main() {
    // Initiate the surrealdb connection
    dbi::initiate_db().await.expect("couldn't initiate DB");

    // Initiate the logger
    logging::initialize_logger().expect("couldn't set up logger");

    info!("info test");
    warn!("warn test");
    error!("error test");

    let data: Data = Data {};
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands_util::help(),
                commands_util::register(),
                commands_util::tag(),
                commands_util::create_tag(),
                commands_util::remove_tag(),
                commands_util::embed(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!r".into()),
                ..Default::default()
            },
            event_handler: |ctx, event, _framework, _data| {
                Box::pin(async move {
                    my_event_handler(ctx, event);
                    Ok(())
                })
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        });

    framework.run().await.unwrap();
}
