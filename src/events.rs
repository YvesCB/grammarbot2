use crate::dbi;
use crate::serenity::Context;
use crate::types::*;
use log::warn;
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
                    ..
                },
                _,
                Some(cpe),
            ) if cpe.guild_emote.id.0 == id.0 => {
                println!("point emote used by {}", reaction.user(ctx).await?.name);
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
                    ..
                },
                _,
                Some(cpe),
            ) if cpe.guild_emote.id.0 == id.0 => {
                println!("point emote used by {}", reaction.user(ctx).await?.name);
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
