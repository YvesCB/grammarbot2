use crate::types::*;
use poise::serenity_prelude::User;

#[poise::command(slash_command, subcommands("user_info"))]
pub async fn user(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Query information about a Discord profile
#[poise::command(category = "User", slash_command)]
pub async fn user_info(
    ctx: Context<'_>,
    #[description = "Discord profile to query information about"] user: Option<User>,
) -> Result<(), Error> {
    match user {
        Some(u) => {
            let response = format!("**Name**: {}\n**Created**: {}", u.name, u.created_at());

            ctx.say(response).await?;
        }
        None => {
            let response = format!(
                "**Name**: {}\n**Created**: {}",
                ctx.author().name,
                ctx.author().created_at()
            );

            ctx.say(response).await?;
        }
    }
    Ok(())
}
