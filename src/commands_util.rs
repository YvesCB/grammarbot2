use crate::dbi;
use crate::types::*;
use poise::serenity_prelude::CacheHttp;
use poise::serenity_prelude::Emoji;
use poise::serenity_prelude::Role;

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

/// Tag parent command
#[poise::command(slash_command, subcommands("remove_tag", "show_tag"))]
pub async fn tag(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Show a pre-written Tag with prepared information.
///
/// Specify the name and the tag will be displayed if it exists.
#[poise::command(slash_command, category = "Tags", rename = "show", guild_only)]
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
#[poise::command(prefix_command, category = "Tags", guild_only)]
pub async fn create_tag(
    ctx: Context<'_>,
    tagname: String,
    #[rest] tagcontent: String,
) -> Result<(), Error> {
    let newtag = Tag {
        name: tagname,
        content: tagcontent,
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

/// Role parent command
#[poise::command(slash_command, subcommands("list_roles", "add_role", "remove_role"))]
pub async fn role(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, category = "Roles", rename = "list", guild_only)]
pub async fn list_roles(ctx: Context<'_>) -> Result<(), Error> {
    let roles = dbi::get_all_roles().await?;
    let roles_string: String = roles
        .iter()
        .map(|r| String::from(format!("{}\n", r.guild_role)))
        .collect();

    ctx.send(|f| {
        f.embed(|f| {
            f.title("Roles available to assign")
                .description(roles_string)
        })
    })
    .await?;

    Ok(())
}

#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Roles",
    rename = "add",
    guild_only
)]
pub async fn add_role(
    ctx: Context<'_>,
    #[description = "Chose a role"] role: Role,
    #[description = "Chose an emote"] emote: Emoji,
    #[description = "Role description"] desc: String,
) -> Result<(), Error> {
    let ur = UserRole {
        guild_role: role,
        emote,
        desc,
    };

    match dbi::add_role(ur, ctx.guild_id()).await {
        Ok(ur) => {
            ctx.say(format!("Role {} added sucessfully!", &ur.guild_role))
                .await?;
        }
        Err(e) => {
            ctx.say(format!("{}", e)).await?;
        }
    };

    Ok(())
}

#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Roles",
    rename = "remove"
)]
pub async fn remove_role(
    ctx: Context<'_>,
    #[description = "Chose a role"] role: Role,
) -> Result<(), Error> {
    Ok(())
}
