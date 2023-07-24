use poise::serenity_prelude::CacheHttp;

async fn autocomplete_tagname<'a>(ctx: crate::Context<'_>, partial: &'a str) -> Vec<String> {
    ctx.data()
        .tags
        .iter()
        .filter(|t| t.name.contains(partial))
        .map(|res| res.name.to_owned())
        .collect()
}

/// Show a pre-written **Tag** with prepared information.
///
/// Specify the name and the tag will be displayed if it exists.
#[poise::command(slash_command)]
pub async fn tag(
    ctx: crate::Context<'_>,
    #[description = "Select a tag"]
    #[autocomplete = "autocomplete_tagname"]
    tag_name: String,
) -> Result<(), crate::Error> {
    let tag = ctx.data().tags.iter().find(|t| t.name == tag_name);
    match tag {
        Some(found_tag) => ctx.say(format!("{}", found_tag.content)).await?,
        None => ctx.say(format!("Tag does not exist!")).await?,
    };
    Ok(())
}

/// Embed test
#[poise::command(slash_command)]
pub async fn embed(ctx: crate::Context<'_>) -> Result<(), crate::Error> {
    ctx.guild()
        .unwrap()
        .channels(ctx.http())
        .await?
        .iter()
        .for_each(|(key, value)| println!("{}: {}", key, value.name()));
    ctx.send(|f| f.embed(|f| f.title("The title").description("The description")))
        .await?;
    Ok(())
}
