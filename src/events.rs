use crate::dbi;
use crate::serenity::Context;
use crate::types::*;
use log::{error, warn};
use poise::serenity_prelude::ReactionType;
use poise::{
    event::Event,
    serenity_prelude::{Message, Reaction},
};

pub async fn my_event_handler(ctx: &Context, event: &Event<'_>) -> Result<(), Error> {
    println!("Got event: {}", event.name());
    match event {
        Event::Message { new_message } => handle_message(new_message),
        Event::ReactionAdd { add_reaction } => handle_add_reaction(ctx, add_reaction).await?,
        Event::ReactionRemove { removed_reaction } => {
            handle_remove_reaction(ctx, removed_reaction).await?
        }
        _ => {}
    };

    Ok(())
}

fn handle_message(msg: &Message) {
    // println!("Someone posted: {:?}", msg);
}

async fn handle_add_reaction(ctx: &Context, reaction: &Reaction) -> Result<(), Error> {
    if reaction.user_id.unwrap() != ctx.cache.current_user_id() {
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
                Reaction {
                    emoji: ReactionType::Custom { id, .. },
                    message_id,
                    channel_id,
                    ..
                },
                _,
                Some(cpe),
            ) if cpe.guild_emote.id.0 == id.0 => {
                let message_reacted_to = ctx
                    .cache
                    .guild_channel(channel_id)
                    .unwrap()
                    .message(ctx, message_id)
                    .await?;
                handle_add_point(&ctx, &reaction, message_reacted_to).await?;
            }
            _ => {}
        }
    };

    Ok(())
}

async fn handle_remove_reaction(ctx: &Context, reaction: &Reaction) -> Result<(), Error> {
    if reaction.user_id.unwrap() != ctx.cache.current_user_id() {
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
                Reaction {
                    emoji: ReactionType::Custom { id, .. },
                    message_id,
                    channel_id,
                    ..
                },
                _,
                Some(cpe),
            ) if cpe.guild_emote.id.0 == id.0 => {
                let message_reacted_to = ctx
                    .cache
                    .guild_channel(channel_id)
                    .unwrap()
                    .message(ctx, message_id)
                    .await?;
                handle_remove_point(&ctx, &reaction, message_reacted_to).await?;
            }

            _ => {}
        }
    };

    Ok(())
}

async fn handle_add_role(
    ctx: &Context,
    reaction: &Reaction,
    cur_roles: Vec<UserRole>,
) -> Result<(), Error> {
    if let ReactionType::Custom {
        animated: _,
        id,
        name: _,
    } = &reaction.emoji
    {
        if let Some(ur) = cur_roles.iter().filter(|ur| ur.emote.id.0 == id.0).next() {
            if let Some(mut member) = ctx
                .cache
                .member(reaction.guild_id.unwrap(), reaction.user_id.unwrap())
            {
                let _ = member.add_role(&ctx.http, ur.guild_role.id).await?;
                warn!(
                    "In {}, events::handle_add_role: Added role {} to member {} with reaction.",
                    reaction.guild_id.unwrap().0,
                    ur.guild_role.name,
                    member.display_name()
                );
                let _ = member
                    .user
                    .direct_message(&ctx, |m| {
                        m.content(format!("The role {} was added to you.", ur.guild_role.name))
                    })
                    .await;
            };
        };
    };

    Ok(())
}

async fn handle_remove_role(
    ctx: &Context,
    reaction: &Reaction,
    cur_roles: Vec<UserRole>,
) -> Result<(), Error> {
    if let ReactionType::Custom {
        animated: _,
        id,
        name: _,
    } = &reaction.emoji
    {
        if let Some(ur) = cur_roles.iter().filter(|ur| ur.emote.id.0 == id.0).next() {
            if let Some(mut member) = ctx
                .cache
                .member(reaction.guild_id.unwrap(), reaction.user_id.unwrap())
            {
                let _ = member.remove_role(&ctx.http, ur.guild_role.id).await?;
                warn!(
                    "In {}, events::handle_remove_role: Removed role {} from member {} with reaction.",
                    reaction.guild_id.unwrap().0,
                    ur.guild_role.name,
                    member.display_name()
                );
                let _ = member
                    .user
                    .direct_message(&ctx, |m| {
                        m.content(format!(
                            "The role {} was removed from you.",
                            ur.guild_role.name
                        ))
                    })
                    .await;
            };
        };
    };

    Ok(())
}

async fn handle_add_point(
    ctx: &Context,
    reaction: &Reaction,
    message: Message,
) -> Result<(), Error> {
    let user = ctx.cache.user(reaction.user_id.unwrap());
    match user {
        Some(u) if u.id.0 != message.author.id.0 => {
            if let Some(author) = ctx.cache.user(message.author.id) {
                let new_user_state =
                    dbi::change_user_points(reaction.guild_id, author, |p| p + 1).await?;
                warn!(
                    "In {}, events::handle_add_point: Added point to {}, new balance {}.",
                    reaction.guild_id.unwrap().0,
                    new_user_state.discord_user.name,
                    new_user_state.grammarpoints
                );
            } else {
                error!("In {}, events::handle_add_point: Attempted to add point to user that is no longer member.", reaction.guild_id.unwrap().0);
            };
        }
        None => {
            error!(
                "In {}, events::handle_add_point: Could not get User struct from reaction.",
                reaction.guild_id.unwrap().0
            );
        }
        _ => {}
    };

    Ok(())
}

async fn handle_remove_point(
    ctx: &Context,
    reaction: &Reaction,
    message: Message,
) -> Result<(), Error> {
    let user = ctx.cache.user(reaction.user_id.unwrap());
    match user {
        Some(u) if u.id.0 != message.author.id.0 => {
            if let Some(author) = ctx.cache.user(message.author.id) {
                let new_user_state =
                    dbi::change_user_points(reaction.guild_id, author, |p| p - 1).await?;
                warn!(
                    "In {}, events::handle_add_point: Removed point from {}, new balance {}.",
                    reaction.guild_id.unwrap().0,
                    new_user_state.discord_user.name,
                    new_user_state.grammarpoints
                );
            } else {
                error!("In {}, events::handle_remove_point: Attempted to add point to user that is no longer member.", reaction.guild_id.unwrap().0);
            };
        }
        None => {
            error!(
                "In {}, events::handle_add_point: Could not get User struct from reaction.",
                reaction.guild_id.unwrap().0
            );
        }
        _ => {}
    };

    Ok(())
}
