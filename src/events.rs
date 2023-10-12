use crate::dbi;
use crate::serenity::Context;
use crate::types::*;
use log::{error, warn};
use poise::serenity_prelude::{Member, ReactionType};
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
        // this gets executed when someone reacts to the role select message
        if let Some((mut member, ur)) = get_member_for_role_edit(ctx, reaction).await? {
            let _ = member.add_role(&ctx.http, ur.guild_role.id).await;
            warn!(
                "In {}, events::handle_add_reaction: Added role {} to member {} with reaction.",
                reaction.guild_id.unwrap().0,
                ur.guild_role.name,
                member.display_name()
            );
        } else {
            warn!(
                "In {}, events::handle_remove_reaction: In process of role_msg update.",
                reaction.guild_id.unwrap().0
            );
        };
    } else {
        // here we handle other cases
    };

    Ok(())
}

async fn handle_remove_reaction(ctx: &Context, reaction: &Reaction) -> Result<(), Error> {
    if reaction.user_id.unwrap() != ctx.cache.current_user_id() {
        // this gets executed when someone reacts to the role select message
        if let Some((mut member, ur)) = get_member_for_role_edit(ctx, reaction).await? {
            let _ = member.remove_role(&ctx.http, ur.guild_role.id).await;
            warn!(
                "In {}, events::handle_remove_reaction: Removed role {} from member {} with reaction.",
                reaction.guild_id.unwrap().0,
                ur.guild_role.name,
                member.display_name()
            );
        } else {
            warn!(
                "In {}, events::handle_remove_reaction: In process of role_msg update.",
                reaction.guild_id.unwrap().0
            );
        };
    };

    Ok(())
}

async fn get_member_for_role_edit(
    ctx: &Context,
    reaction: &Reaction,
) -> Result<Option<(Member, UserRole)>, Error> {
    // first we check if we're in a guild and if the cache is available to avoid any error
    if let (Some(guild_id), Some(user_id)) = (reaction.guild_id, reaction.user_id) {
        // get the roles currently set in the db on the current guild as user assignable
        let cur_roles = dbi::get_all_roles(Some(guild_id)).await?;
        let cur_role_msg = dbi::get_role_message(Some(guild_id)).await?;

        if let Some(cur_role_msg) = cur_role_msg {
            if let Some(guild_msg) = cur_role_msg.guild_message {
                // if the reaction is of the emote type
                if let ReactionType::Custom {
                    animated: _,
                    id,
                    name: _,
                } = &reaction.emoji
                {
                    if guild_msg.id == reaction.message_id {
                        if let Some(ur) = cur_roles
                            .into_iter()
                            .filter(|ur| ur.emote.id.0 == id.0)
                            .next()
                        {
                            if let Some(member) = ctx.cache.member(guild_id, user_id) {
                                return Ok(Some((member, ur)));
                            } else {
                                error!("In {}, events::get_member_for_role_edit: Can't access members.", guild_id.0);
                                return Ok(None);
                            };
                        };
                    }
                }
            }
        }
    }

    Ok(None)
}
