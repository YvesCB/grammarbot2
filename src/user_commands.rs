use crate::constants;
use crate::dbi;
use crate::types::*;
use poise::serenity_prelude::Colour;
use poise::serenity_prelude::Member;

#[poise::command(slash_command, subcommands("user_info"))]
pub async fn user(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Query information about a Discord profile
#[poise::command(category = "User", slash_command, guild_only)]
pub async fn user_info(
    ctx: Context<'_>,
    #[description = "Discord profile to query information about"] user: Option<Member>,
) -> Result<(), Error> {
    match user {
        Some(m) => {
            let mut points: u32 = 0;
            let myuser = dbi::get_user_data(ctx.guild_id(), m.user.id.0).await;
            if let Ok(mu) = myuser {
                points = mu.grammarpoints;
            }
            let fields = vec![
                ("ID", m.user.id.to_string(), false),
                (
                    "Joined at",
                    m.joined_at.unwrap().format("%d. %b %Y %H:%M").to_string(),
                    false,
                ),
                (
                    "Created at",
                    m.user.created_at().format("%d. %b %Y %H:%M").to_string(),
                    false,
                ),
                ("Points", points.to_string(), false),
            ];
            ctx.send(|f| {
                f.embed(|f| {
                    f.title(&m.user.name)
                        .thumbnail(match &m.user.avatar_url() {
                            Some(url) => url,
                            None => constants::DEFAULT_AVATAR,
                        })
                        .fields(fields.into_iter())
                        .colour(Colour::BLUE)
                        .footer(|f| {
                            f.text(format!("Requested by: {}", ctx.author().name))
                                .icon_url(
                                    ctx.serenity_context()
                                        .cache
                                        .current_user()
                                        .avatar_url()
                                        .unwrap(),
                                )
                        })
                })
            })
            .await?;
        }
        None => {
            let m = ctx.author_member().await.unwrap();
            let mut points: u32 = 0;
            let myuser = dbi::get_user_data(ctx.guild_id(), m.user.id.0).await;
            if let Ok(mu) = myuser {
                points = mu.grammarpoints;
            }
            let fields = vec![
                ("ID", m.user.id.to_string(), false),
                (
                    "Joined at",
                    m.joined_at.unwrap().format("%d. %b %Y %H:%M").to_string(),
                    false,
                ),
                (
                    "Created at",
                    m.user.created_at().format("%d. %b %Y %H:%M").to_string(),
                    false,
                ),
                ("Points", points.to_string(), false),
            ];
            ctx.send(|f| {
                f.embed(|f| {
                    f.title(&m.user.name)
                        .thumbnail(match &m.user.avatar_url() {
                            Some(url) => url,
                            None => constants::DEFAULT_AVATAR,
                        })
                        .fields(fields.into_iter())
                        .colour(Colour::BLUE)
                        .footer(|f| {
                            f.text(format!("Requested by: {}", ctx.author().name))
                                .icon_url(
                                    ctx.serenity_context()
                                        .cache
                                        .current_user()
                                        .avatar_url()
                                        .unwrap(),
                                )
                        })
                })
            })
            .await?;
        }
    }
    Ok(())
}
