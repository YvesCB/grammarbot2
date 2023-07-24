// use poise::serenity_prelude as serenity;

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
