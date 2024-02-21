use crate::dbi;
use crate::types::*;
use poise::serenity_prelude as serenity;

/// Role parent command
///
/// This bot allows for assigning user roles via reactions to a message. With the commands in this
/// category, you can create a custom message, assign different roles different emotes and then
/// post the message. From that point, the bot will assign roles to users that react with the
/// emote to said message and it'll remove it when the reaction is removed.
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_ROLES",
    subcommands(
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
    #[description = "Role on this server"] role: serenity::Role,
    #[description = "Emote for the role"] emote: serenity::Emoji,
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
    #[description = "Role to remove"] role: serenity::Role,
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

/// Resets all the roles
///
/// This command removes all the roles from the role message this is mostly for the
/// purpose of getting rid of roles that have already been deleted from the server.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Roles",
    rename = "reset",
    guild_only
)]
pub async fn reset_roles(ctx: Context<'_>) -> Result<(), Error> {
    let removed_roles = dbi::remove_all_roles(ctx.guild_id()).await?;
    let removed_roles: String = removed_roles
        .iter()
        .map(|r| r.guild_role.name.to_owned())
        .collect::<Vec<String>>()
        .join(" ");

    if removed_roles.len() > 0 {
        ctx.say(removed_roles).await?;
    }

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
    #[description = "Desired message text"] msg: String,
) -> Result<(), Error> {
    match dbi::set_role_message(msg, ctx.author(), ctx.guild_id()).await {
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
    let cur_roles = dbi::get_all_roles(ctx.guild_id()).await?;
    let roles_string: String = cur_roles
        .iter()
        .map(|r| String::from(format!("{} {}: {}\n", r.emote, r.guild_role, r.desc)))
        .collect();
    match dbi::get_role_message(ctx.guild_id()).await? {
        Some(msg) => {
            ctx.send(
                poise::CreateReply::default().embed(
                    serenity::CreateEmbed::default()
                        .title("Current role message")
                        .description(&msg.messagetext)
                        .field("Roles", roles_string, false)
                        .field(
                            "Message link",
                            format!(
                                "{}",
                                match msg.guild_message {
                                    Some(msg) => msg.link(),
                                    None => "None".to_string(),
                                }
                            ),
                            true,
                        )
                        .field("Is active", msg.active.to_string(), true)
                        .colour(serenity::Colour::BLUE)
                        .footer(
                            serenity::CreateEmbedFooter::new(format!(
                                "Requested by: {}",
                                ctx.author().name
                            )), // .icon_url(
                                //     ctx.serenity_context()
                                //         .cache
                                //         .current_user()
                                //         .avatar_url()
                                //         .unwrap(),
                                // ),
                        ),
                ),
            )
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
    #[description = "Channel to post in"] channel: serenity::Channel,
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
            dbi::set_active_role_message(&msg, sent_message, true, ctx.author(), ctx.guild_id())
                .await?;

            ctx.say("Message posted sucessfully.").await?;
        }
        _ => {
            ctx.say("Make sure that a role message is set and that at least one role was selected as a user assignable role.").await?;
        }
    };

    Ok(())
}

/// Set the active state of the role message
///
/// This command let's you set the active state of the role message. If it is set to true, the bot
/// will assign roles upon reaction, and if it's set to false, it won't.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Roles",
    rename = "set_active",
    guild_only
)]
pub async fn activate_msg_role(
    ctx: Context<'_>,
    #[description = "New state"] state: bool,
) -> Result<(), Error> {
    if let Some(cur_msg) = dbi::get_role_message(ctx.guild_id()).await?.as_ref() {
        if let Some(guild_msg) = &cur_msg.guild_message {
            dbi::set_active_role_message(
                cur_msg,
                guild_msg.to_owned(),
                state,
                ctx.author(),
                ctx.guild_id(),
            )
            .await?;
        } else {
            ctx.say("Role message exists but it's not posted anywhere. Post it to a channel first using the `/role post` command first before attempting to change its status.").await?;
        }
    } else {
        ctx.say("No role message created. Create one and post it first before attempting to change its status.").await?;
    }

    Ok(())
}
