use crate::dbi;
use crate::types::*;
use log::warn;
use poise::serenity_prelude::Channel;
use poise::serenity_prelude::{CacheHttp, Emoji, Role};

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
#[poise::command(
    slash_command,
    subcommands(
        "list_roles",
        "add_role",
        "remove_role",
        "show_msg_role",
        "set_msg_role",
        "post_msg_role",
    )
)]
pub async fn role(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Lists all user assignable roles
///
/// This command will post the list of all the roles that users can assign to themselves along with
/// their description.
#[poise::command(slash_command, category = "Roles", rename = "list", guild_only)]
pub async fn list_roles(ctx: Context<'_>) -> Result<(), Error> {
    let roles = dbi::get_all_roles(ctx.guild_id()).await?;
    let roles_string: String = roles
        .iter()
        .map(|r| String::from(format!("{} {}: {}\n", r.emote, r.guild_role, r.desc)))
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

/// Adds a role as user assignable
///
/// With this command you can add a role to the list of roles that users can assign to themselves.
/// Additionally an emote is associated with the role, as well as a description which explains what
/// the function of the role is.
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

/// Removes a role from the user assignable roles
///
/// This command can be used to remove a role from the list of user assignable roles, so that it
/// can no longer be assigne by the users.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Roles",
    rename = "remove",
    guild_only
)]
pub async fn remove_role(
    ctx: Context<'_>,
    #[description = "Chose a role"] role: Role,
) -> Result<(), Error> {
    let user_role = dbi::get_role(role.id.to_string(), ctx.guild_id()).await?;

    match dbi::remove_role(user_role, ctx.guild_id()).await {
        Ok(ur) => {
            ctx.say(format!(
                "Role {} successfully removed from list.",
                ur.guild_role
            ))
            .await?;
        }
        Err(e) => {
            ctx.say(format!("{}", e)).await?;
        }
    };

    Ok(())
}

/// Sets the text for the role selection message
///
/// With this command the text shown in the role selection message can be set. This text will then
/// show up on the message which lets the users select their roles.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Roles",
    rename = "message_set",
    guild_only
)]
pub async fn set_msg_role(
    ctx: Context<'_>,
    #[description = "Set the desired message text"] msg: String,
) -> Result<(), Error> {
    match dbi::set_role_message(msg, ctx.guild_id()).await {
        Ok(_) => {
            ctx.say(format!("Role message set successfully.")).await?;
        }
        Err(e) => {
            ctx.say(format!("{}", e)).await?;
        }
    };

    Ok(())
}

/// Shows the currently set role message
///
/// With this commmand you can print the currently set up role message.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Roles",
    rename = "message_show",
    guild_only
)]
pub async fn show_msg_role(ctx: Context<'_>) -> Result<(), Error> {
    match dbi::get_role_message(ctx.guild_id()).await? {
        Some(msg) => {
            ctx.send(|f| {
                f.embed(|f| {
                    f.title("Current role message")
                        .description(&msg.messagetext)
                        .field(
                            "Message link",
                            format!(
                                "{:?}",
                                match msg.guild_message {
                                    Some(msg) => msg.link(),
                                    None => "None".to_string(),
                                }
                            ),
                            true,
                        )
                        .field("Is active", msg.active, true)
                })
            })
            .await?;
        }
        None => {
            ctx.say(format!("No role message set on this server"))
                .await?;
        }
    };

    Ok(())
}

/// Post the role message in the specified chat
///
/// This command will attempt to post the full role message in the specified channel. It will
/// contain the role message and all the roles with their correspoding emotes. The reactions will
/// be aded automatically and from that point on, the reaction roles will be active.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Roles",
    rename = "post",
    guild_only
)]
pub async fn post_msg_role(
    ctx: Context<'_>,
    #[description = "Chose channel"] channel: Channel,
) -> Result<(), Error> {
    // First we get the components we need to build the message for the current server
    let cur_message = dbi::get_role_message(ctx.guild_id()).await?;
    let cur_roles = dbi::get_all_roles(ctx.guild_id()).await?;

    match (cur_message, &cur_roles) {
        (Some(msg), roles) if roles.len() > 0 => {
            // At this point we know we have a message and a list of roles that has a least one
            // role
            let role_list: String = roles
                .iter()
                .map(|r| String::from(format!("{} {}: {}\n", r.emote, r.guild_role, r.desc)))
                .collect();
            let message: String = String::from(format!(
                "# Reaction roles\n\
{}\n\
## Available roles\n\
{}",
                msg.messagetext, role_list
            ));

            let sent_message = channel.id().say(ctx.http(), message).await?;
            for role in cur_roles.iter() {
                sent_message.react(ctx, role.emote.to_owned()).await?;
            }
            dbi::activate_role_message(&msg, sent_message, ctx.guild_id()).await?;

            ctx.say("Message posted sucessfully.").await?;
        }
        _ => {
            ctx.say("Make sure that a role message is set and that at least one role was selected as a user assignable role.").await?;
        }
    };

    Ok(())
}

/// Grammarpoint parent command
#[poise::command(slash_command, subcommands("emote_set",))]
pub async fn points(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the emote for the grammar points
///
/// This command is used to set an emote from the guild to be the GrammarPoint emote. Whenever a
/// user that isn't the author of the message reacts with said emote to a message, one GrammarPoint
/// will be added to the authors Points.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Points",
    guild_only
)]
pub async fn emote_set(
    ctx: Context<'_>,
    #[description = "Chose channel"] emote: Emoji,
) -> Result<(), Error> {
    dbi::set_point_emote(&emote, ctx.guild_id()).await?;

    ctx.say(format!("Set the new point emote to: {}", emote))
        .await?;
    Ok(())
}
