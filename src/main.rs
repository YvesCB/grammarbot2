use commands_util::{embed, register, tag};
use events::my_event_handler;
use poise::serenity_prelude as serenity;
use surrealdb::engine::remote::ws::Client as SurrealClient;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use types::*;

mod commands_util;
mod events;
mod types;

static DB: Surreal<SurrealClient> = Surreal::init();

async fn initiate_db() -> surrealdb::Result<()> {
    DB.connect::<Ws>("localhost:8000").await?;
    println!("Connected to DB at localhost:8000");

    let dbuser = std::env::var("SURREAL_USER").expect("missing SURREAL_USER");
    let dbpass = std::env::var("SURREAL_PASS").expect("missing SURREAL_PASS");

    DB.signin(Root {
        username: &dbuser,
        password: &dbpass,
    })
    .await?;
    println!("Signed into DB");

    DB.use_ns("discordbot").use_db("grammarbot").await?;
    println!("Using ns discordbot and db grammarbot");

    Ok(())
}

#[allow(dead_code)]
async fn db_test() -> surrealdb::Result<()> {
    let del_tags: Result<Vec<Tag>, surrealdb::Error> = DB.delete("tag").await;
    match del_tags {
        Ok(tags) => println!("Deleted: {:?}", tags),
        Err(_) => println!("Table empty, nothing deleted."),
    }
    let del_users: Result<Vec<MyUser>, surrealdb::Error> = DB.delete("user").await;
    match del_users {
        Ok(users) => println!("Deleted: {:?}", users),
        Err(_) => println!("Table empty, nothing deleted."),
    }

    let created: Record = DB
        .create("tag")
        .content(Tag {
            name: "hello".to_string(),
            content: "Content of the hello tag!".to_string(),
        })
        .await?;
    println!("{:?}", created);

    let created: Record = DB
        .create("tag")
        .content(Tag {
            name: "goodbye".to_string(),
            content: "Content of the goodbye tag!".to_string(),
        })
        .await?;
    println!("{:?}", created);

    let created: Record = DB
        .create("user")
        .content(MyUser {
            name: "yves".to_string(),
            userid: "123".to_string(),
        })
        .await?;
    println!("{:?}", created);

    //now get all tags
    let tags: Vec<Tag> = DB.select("tag").await?;
    println!("{:?}", tags);

    // and the user
    let users: Vec<MyUser> = DB.select("user").await?;
    println!("{:?}", users);

    Ok(())
}

#[tokio::main]
async fn main() {
    // Initiate the surrealdb connection
    initiate_db().await.expect("Couldn't initiate DB");

    let data: Data = Data {};
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![embed(), register()],
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
