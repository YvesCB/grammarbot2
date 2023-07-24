use commands_extra::{embed, tag};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Tag {
    name: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    tags: Vec<Tag>,
} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, crate::Data, Error>;

mod commands_extra;

#[tokio::main]
async fn main() {
    let data: Data =
        serde_json::from_str(&fs::read_to_string("tag_test_data.json").unwrap()).unwrap();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![tag(), embed()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        });

    framework.run().await.unwrap();
}
