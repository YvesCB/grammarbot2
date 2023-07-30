use crate::types::*;
use poise::serenity_prelude::CacheHttp;

// async fn autocomplete_tagname<'a>(ctx: Context<'_>, partial: &'a str) -> Vec<String> {
//     ctx.data()
//         .tags
//         .iter()
//         .filter(|t| t.name.contains(partial))
//         .map(|res| res.name.to_owned())
//         .collect()
// }

/// Register and unregister commands
#[poise::command(slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    // use this for reference when creating buttons
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Show a pre-written Tag with prepared information.
///
/// Specify the name and the tag will be displayed if it exists.
#[poise::command(slash_command)]
pub async fn tag(
    ctx: Context<'_>,
    #[description = "Select a tag"]
    // #[autocomplete = "autocomplete_tagname"]
    tag_name: String,
) -> Result<(), Error> {
    todo!();
}

/// Embed test
#[poise::command(slash_command)]
pub async fn embed(ctx: Context<'_>) -> Result<(), Error> {
    let channels: String = ctx
        .guild()
        .unwrap()
        .channels(ctx.http())
        .await?
        .iter()
        .map(|(key, value)| String::from(format!("{}: {}\n", key, value.name())))
        .collect();
    ctx.send(|f| f.embed(|f| f.title("The title").description(channels)))
        .await?;
    Ok(())
}
