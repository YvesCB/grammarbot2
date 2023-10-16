use crate::dbi;
use crate::types::*;

async fn autocomplete_tagname<'a>(ctx: Context<'_>, partial: &'a str) -> Vec<String> {
    let tags = dbi::get_all_tags(ctx.guild_id()).await;
    match tags {
        Ok(t) => t
            .iter()
            .filter(|t| t.name.contains(partial))
            .map(|res| res.name.to_owned())
            .collect(),
        Err(_) => vec![],
    }
}

/// Tag parent command
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_MESSAGES",
    subcommands("remove_tag")
)]
pub async fn tags(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Show a pre-written Tag with prepared information.
///
/// Specify the name and the tag will be displayed if it exists.
#[poise::command(slash_command, category = "Tags", rename = "tag", guild_only)]
pub async fn show_tag(
    ctx: Context<'_>,
    #[description = "Select a tag"]
    #[autocomplete = "autocomplete_tagname"]
    tagname: String,
) -> Result<(), Error> {
    let tag = dbi::get_tag(&tagname, ctx.guild_id()).await;
    match tag {
        Ok(t) => {
            ctx.say(&t.content).await?;
        }
        Err(e) => {
            ctx.say(format!("{:?}", e)).await?;
        }
    };

    Ok(())
}

/// Create a tag by specifying the name, followed by the content.
///
/// The name needs to be one word without spaces. Everything after the name will be considered part
/// of the content
#[poise::command(
    prefix_command,
    required_permissions = "MANAGE_MESSAGES",
    category = "Tags",
    guild_only
)]
pub async fn create_tag(
    ctx: Context<'_>,
    tagname: String,
    #[rest] tagcontent: String,
) -> Result<(), Error> {
    let newtag = Tag {
        name: tagname,
        content: tagcontent,
        creator: ctx.author().to_owned(),
    };

    match dbi::create_tag(newtag, ctx.guild_id()).await {
        Ok(t) => {
            ctx.say(format!("Tag {} created sucessfully!", &t.name))
                .await?;
        }
        Err(e) => {
            ctx.say(format!("{}", e)).await?;
        }
    }

    Ok(())
}

/// Removes a tag
///
/// This command can only be used by people with the manage messages permission.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES",
    category = "Tags",
    rename = "remove",
    guild_only
)]
pub async fn remove_tag(
    ctx: Context<'_>,
    #[description = "Chose name"]
    #[autocomplete = "autocomplete_tagname"]
    tagname: String,
) -> Result<(), Error> {
    match dbi::remove_tag(&tagname, ctx.guild_id()).await {
        Ok(t) => {
            ctx.say(format!("Tag {} removed sucessfully!", t.name))
                .await?
        }
        Err(e) => ctx.say(format!("{}", e)).await?,
    };

    Ok(())
}
