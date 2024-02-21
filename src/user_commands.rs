use crate::constants;
use crate::dbi;
use crate::types::*;
use poise::serenity_prelude as serenity;

/// Parent command for the User category
///
/// Displaying user information and potentially more in the future.
#[poise::command(slash_command, subcommands("user_info"))]
pub async fn user(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Query information about a Discord profile
///
/// This command will display information about the user such as when then joined the server, when
/// they created their account and also how many points they have collected on this server.
#[poise::command(category = "User", slash_command, guild_only)]
pub async fn user_info(
    ctx: Context<'_>,
    #[description = "User to get info about"] user: Option<serenity::Member>,
) -> Result<(), Error> {
    match user {
        Some(m) => {
            let mut points: u32 = 0;
            let myuser = dbi::get_user_data(ctx.guild_id(), m.user.id.get()).await;
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
            ctx.send(
                poise::CreateReply::default().embed(
                    serenity::CreateEmbed::default()
                        .title(&m.user.name)
                        .thumbnail(match &m.user.avatar_url() {
                            Some(url) => url,
                            None => constants::DEFAULT_AVATAR,
                        })
                        .fields(fields.into_iter())
                        .colour(serenity::Colour::BLUE)
                        .footer(
                            serenity::CreateEmbedFooter::new(format!(
                                "Requested by: {}",
                                ctx.author().name
                            ))
                            .icon_url(
                                ctx.serenity_context()
                                    .cache
                                    .current_user()
                                    .avatar_url()
                                    .unwrap(),
                            ),
                        ),
                ),
            )
            .await?;
        }
        None => {
            let m = ctx.author_member().await.unwrap();
            let mut points: u32 = 0;
            let myuser = dbi::get_user_data(ctx.guild_id(), m.user.id.get()).await;
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
            ctx.send(
                poise::CreateReply::default().embed(
                    serenity::CreateEmbed::default()
                        .title(&m.user.name)
                        .thumbnail(match &m.user.avatar_url() {
                            Some(url) => url,
                            None => constants::DEFAULT_AVATAR,
                        })
                        .fields(fields.into_iter())
                        .colour(serenity::Colour::BLUE)
                        .footer(
                            serenity::CreateEmbedFooter::new(format!(
                                "Requested by: {}",
                                ctx.author().name
                            ))
                            .icon_url(
                                ctx.serenity_context()
                                    .cache
                                    .current_user()
                                    .avatar_url()
                                    .unwrap(),
                            ),
                        ),
                ),
            )
            .await?;
        }
    }
    Ok(())
}
