use commands_util::{embed, register, tag};
use events::my_event_handler;
use poise::serenity_prelude as serenity;
use std::fs;
use types::*;

mod commands_util;
mod events;
mod types;

#[tokio::main]
async fn main() {
    let data: Data =
        serde_json::from_str(&fs::read_to_string("tag_test_data.json").unwrap()).unwrap();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![tag(), embed(), register()],
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
