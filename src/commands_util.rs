use crate::dbi;
use crate::types::*;
use poise::serenity_prelude::CacheHttp;

async fn autocomplete_tagname<'a>(_ctx: Context<'_>, partial: &'a str) -> Vec<String> {
    let tags = dbi::get_all_tags().await;
    match tags {
        Ok(t) => t
            .iter()
            .filter(|t| t.name.contains(partial))
            .map(|res| res.name.to_owned())
            .collect(),
        Err(_) => vec![],
    }
}

/// Register and unregister commands
///
/// This command can be used to register and unregister commands of this bot.
/// Only the owner can use this command.
#[poise::command(slash_command, owners_only, category = "Admins")]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    // use this for reference when creating buttons
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Print the help text
///
/// This prints the help text which lists all commands.
/// You can also specify an optional parameter to get info on a specific command.
#[poise::command(prefix_command, track_edits, slash_command, category = "Various")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type ?help command for more info on a command.
You can edit your message to the bot and the bot will edit its response.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Embed test
#[poise::command(slash_command, category = "Various")]
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

/// Show a pre-written Tag with prepared information.
///
/// Specify the name and the tag will be displayed if it exists.
#[poise::command(slash_command, category = "Tags")]
pub async fn tag(
    ctx: Context<'_>,
    #[description = "Select a tag"]
    #[autocomplete = "autocomplete_tagname"]
    tagname: String,
) -> Result<(), Error> {
    let tag = dbi::get_tag(&tagname).await;
    match tag {
        Ok(t) => ctx.say(t.content).await?,
        Err(e) => ctx.say(format!("{:?}", e)).await?,
    };

    Ok(())
}

/// Create a tag by specifying the name, followed by the content.
///
/// The name needs to be one word without spaces. Everything after the name will be considered part
/// of the content
#[poise::command(prefix_command, category = "Tags")]
pub async fn create_tag(ctx: Context<'_>, #[rest] tag: String) -> Result<(), Error> {
    println!("{}", &tag);
    let mut words = tag.split_whitespace();

    // first word is tagname
    let tagname = words.next();

    // the remaining words are the content
    let tagcontent: String = words.collect::<Vec<_>>().join(" ");

    let newtag = match (tagname, tagcontent.len()) {
        (Some(name), len) if len > 0 => Some(Tag {
            name: name.to_string(),
            content: tagcontent,
        }),
        _ => None,
    };

    match newtag {
        Some(tag) => match dbi::create_tag(&tag).await {
            Ok(()) => {
                ctx.say(format!("Tag {} created sucessfully!", &tag.name))
                    .await?
            }
            Err(e) => ctx.say(format!("{}", e)).await?,
        },
        None => {
            ctx.say(
                "**Error:** After the tag name, you must have at least one character of content!",
            )
            .await?
        }
    };

    Ok(())
}

/// Removes a tag
///
/// This command can only be used by people with the manage messages permission.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES",
    category = "Tags"
)]
pub async fn remove_tag(
    ctx: Context<'_>,
    #[description = "Chose name"]
    #[autocomplete = "autocomplete_tagname"]
    tagname: String,
) -> Result<(), Error> {
    match dbi::remove_tag(&tagname).await {
        Ok(t) => {
            ctx.say(format!("Tag {} removed sucessfully!", t.name))
                .await?
        }
        Err(e) => ctx.say(format!("{}", e)).await?,
    };

    Ok(())
}
