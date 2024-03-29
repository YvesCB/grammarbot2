use db_interactions as dbi;
use poise::serenity_prelude as serenity;

use log4rs;

use events::my_event_handler;
use types::*;

mod commands_util;
mod constants;
mod db_interactions;
mod embed_tools;
mod events;
mod point_commands;
mod role_commands;
mod tag_commands;
mod types;
mod user_commands;

#[tokio::main]
async fn main() {
    // Initiate the surrealdb connection
    log4rs::init_file("logging_config.yaml", Default::default()).unwrap();

    dbi::initiate_db().await.expect("couldn't initiate DB");

    let data: Data = Data {};
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::privileged();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands_util::help(),
                commands_util::register(),
                tag_commands::tags(),
                tag_commands::create_tag(),
                role_commands::role(),
                point_commands::points(),
                user_commands::user_info(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(constants::BOT_PREFIX.into()),
                ..Default::default()
            },
            event_handler: |ctx, event, _framework, _data| Box::pin(my_event_handler(ctx, event)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
