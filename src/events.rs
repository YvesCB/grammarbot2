use crate::dbi;
use crate::serenity::Context;
use crate::types::*;
use log::{error, warn};
use poise::serenity_prelude::{self as serenity, CacheHttp};

pub async fn my_event_handler(ctx: &Context, event: &serenity::FullEvent) -> Result<(), Error> {
    // println!("Got event: {}", event.name().unwrap());
    match event {
        // Event::Message { new_message } => handle_message(ctx, new_message).await?,
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            handle_add_reaction(ctx, &add_reaction).await?
        }
        serenity::FullEvent::ReactionRemove { removed_reaction } => {
            handle_remove_reaction(ctx, &removed_reaction).await?
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            todo!()
        }
        _ => {}
    };

    Ok(())
}

async fn handle_add_reaction(ctx: &Context, reaction: &serenity::Reaction) -> Result<(), Error> {
    if reaction.user_id.unwrap() != ctx.cache.current_user().id {
        let cur_role_msg = dbi::get_role_message(reaction.guild_id).await?;
        let cur_point_emote = dbi::get_point_data(reaction.guild_id).await?;
        // we only handle role reactions if the messge exists in the first place

        match (reaction, cur_role_msg, cur_point_emote) {
            // this is if it's a reaction to the role message
            // we check if the message reacted to is the same id as the guild message saved in the
            // db
            // we also make sure that we have a role message in the db and that a guild message was
            // posted
            (
                rct,
                Some(RoleMessage {
                    guild_message: Some(gm),
                    ..
                }),
                _,
            ) if rct.message_id == gm.id => {
                let cur_roles = dbi::get_all_roles(reaction.guild_id).await?;
                handle_add_role(ctx, reaction, cur_roles).await?;
            }

            // this is if the reaction is a point emote
            // we check if the id of the emote in the reaction matches the id of the emote saved to
            // the db
            (
                serenity::Reaction {
                    emoji: serenity::ReactionType::Custom { id, .. },
                    ..
                },
                _,
                Some(cpe),
            ) if cpe.guild_emote.id.get() == id.get() => {
                let message_reacted_to = reaction.message(ctx).await?;
                handle_add_point(&ctx, &reaction, message_reacted_to).await?;
            }
            _ => {}
        }
    };

    Ok(())
}

async fn handle_remove_reaction(ctx: &Context, reaction: &serenity::Reaction) -> Result<(), Error> {
    if reaction.user_id.unwrap() != ctx.cache.current_user().id {
        let cur_role_msg = dbi::get_role_message(reaction.guild_id).await?;
        let cur_point_emote = dbi::get_point_data(reaction.guild_id).await?;
        // we only handle role reactions if the messge exists in the first place

        match (reaction, cur_role_msg, cur_point_emote) {
            // this is if it's a reaction to the role message
            // we check if the message reacted to is the same id as the guild message saved in the
            // db
            // we also make sure that we have a role message in the db and that a guild message was
            // posted
            (
                rct,
                Some(RoleMessage {
                    guild_message: Some(gm),
                    ..
                }),
                _,
            ) if rct.message_id == gm.id => {
                let cur_roles = dbi::get_all_roles(reaction.guild_id).await?;
                handle_remove_role(ctx, reaction, cur_roles).await?;
            }

            // this is if the reaction is a point emote
            // we check if the id of the emote in the reaction matches the id of the emote saved to
            // the db
            (
                serenity::Reaction {
                    emoji: serenity::ReactionType::Custom { id, .. },
                    ..
                },
                _,
                Some(cpe),
            ) if cpe.guild_emote.id.get() == id.get() => {
                let message_reacted_to = reaction.message(ctx).await?;
                handle_remove_point(&ctx, &reaction, message_reacted_to).await?;
            }

            _ => {}
        }
    };

    Ok(())
}

async fn handle_add_role(
    ctx: &Context,
    reaction: &serenity::Reaction,
    cur_roles: Vec<UserRole>,
) -> Result<(), Error> {
    if let serenity::ReactionType::Custom {
        animated: _,
        id,
        name: _,
    } = &reaction.emoji
    {
        if let Some(ur) = cur_roles
            .iter()
            .filter(|ur| ur.emote.id.get() == id.get())
            .next()
        {
            if let Ok(member) = reaction
                .guild_id
                .unwrap()
                .member(ctx.http(), reaction.user_id.unwrap())
                .await
            {
                let _ = member.add_role(&ctx.http, ur.guild_role.id).await?;
                warn!(
                    "In {}, events::handle_add_role: Added role {} to member {} with reaction.",
                    reaction.guild_id.unwrap().get(),
                    ur.guild_role.name,
                    member.display_name()
                );
                let _ = member
                    .user
                    .direct_message(
                        &ctx,
                        serenity::CreateMessage::new()
                            .content(format!("The role {} was added to you.", ur.guild_role.name)),
                    )
                    .await;
            };
        };
    };

    Ok(())
}

async fn handle_remove_role(
    ctx: &Context,
    reaction: &serenity::Reaction,
    cur_roles: Vec<UserRole>,
) -> Result<(), Error> {
    if let serenity::ReactionType::Custom {
        animated: _,
        id,
        name: _,
    } = &reaction.emoji
    {
        if let Some(ur) = cur_roles
            .iter()
            .filter(|ur| ur.emote.id.get() == id.get())
            .next()
        {
            if let Ok(member) = reaction
                .guild_id
                .unwrap()
                .member(ctx.http(), reaction.user_id.unwrap())
                .await
            {
                let _ = member.remove_role(&ctx.http, ur.guild_role.id).await?;
                warn!(
                    "In {}, events::handle_remove_role: Removed role {} from member {} with reaction.",
                    reaction.guild_id.unwrap().get(),
                    ur.guild_role.name,
                    member.display_name()
                );
                let _ = member
                    .user
                    .direct_message(
                        &ctx,
                        serenity::CreateMessage::new().content(format!(
                            "The role {} was removed from you.",
                            ur.guild_role.name
                        )),
                    )
                    .await;
            };
        };
    };

    Ok(())
}

async fn handle_add_point(
    ctx: &Context,
    reaction: &serenity::Reaction,
    message: serenity::Message,
) -> Result<(), Error> {
    let user = reaction.user(ctx).await;
    match user {
        Ok(u) if u.id.get() != message.author.id.get() => {
            if let Ok(author) = reaction
                .guild_id
                .unwrap()
                .member(ctx, message.author.id)
                .await
            {
                let new_user_state =
                    dbi::change_user_points(reaction.guild_id, author.user, |p| p + 1).await?;
                warn!(
                    "In {}, events::handle_add_point: Added point to {}, new balance {}.",
                    reaction.guild_id.unwrap().get(),
                    new_user_state.discord_user.name,
                    new_user_state.grammarpoints
                );
            } else {
                error!("In {}, events::handle_add_point: Attempted to add point to user that is no longer member.", reaction.guild_id.unwrap().get());
            };
        }
        Err(_) => {
            error!(
                "In {}, events::handle_add_point: Could not get User struct from reaction.",
                reaction.guild_id.unwrap().get()
            );
        }
        _ => {}
    };

    Ok(())
}

async fn handle_remove_point(
    ctx: &Context,
    reaction: &serenity::Reaction,
    message: serenity::Message,
) -> Result<(), Error> {
    let user = reaction.user(ctx).await;
    match user {
        Ok(u) if u.id.get() != message.author.id.get() => {
            if let Ok(author) = reaction
                .guild_id
                .unwrap()
                .member(ctx, message.author.id)
                .await
            {
                let new_user_state =
                    dbi::change_user_points(reaction.guild_id, author.user, |p| p - 1).await?;
                warn!(
                    "In {}, events::handle_add_point: Removed point from {}, new balance {}.",
                    reaction.guild_id.unwrap().get(),
                    new_user_state.discord_user.name,
                    new_user_state.grammarpoints
                );
            } else {
                error!("In {}, events::handle_remove_point: Attempted to add point to user that is no longer member.", reaction.guild_id.unwrap().get());
            };
        }
        Err(_) => {
            error!(
                "In {}, events::handle_add_point: Could not get User struct from reaction.",
                reaction.guild_id.unwrap().get()
            );
        }
        _ => {}
    };

    Ok(())
}
