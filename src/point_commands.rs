use crate::dbi;
use crate::embed_tools::paginate_with_embeds;
use crate::types::*;
use crate::user_commands::user;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::{Colour, Emoji};

/// Grammarpoint parent command
#[poise::command(
    slash_command,
    default_member_permissions = "ADMINISTRATOR",
    subcommands("emote_set", "emote_stats", "leaderboard")
)]
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
    required_permissions = "ADMINISTRATOR",
    category = "Points",
    guild_only
)]
pub async fn emote_set(
    ctx: Context<'_>,
    #[description = "Chose channel"] emote: Emoji,
) -> Result<(), Error> {
    dbi::set_point_emote(&emote, ctx.author(), ctx.guild_id()).await?;

    ctx.say(format!("Set the new point emote to: {}", emote))
        .await?;
    Ok(())
}

/// Show the status of the point functionality
///
/// This command will display the information about the Point System. Which emote is set, who set
/// it and if the point system is active.
#[poise::command(
    slash_command,
    required_permissions = "ADMINISTRATOR",
    category = "Points",
    guild_only,
    rename = "stats"
)]
pub async fn emote_stats(ctx: Context<'_>) -> Result<(), Error> {
    let cur_points = dbi::get_point_data(ctx.guild_id()).await?;

    match cur_points {
        Some(pointsdata) => {
            ctx.send(|f| {
                f.embed(|e| {
                    e.title("Points system info")
                        .description("Points can be given to users by other users by reacting to their messages with the point emote.")
                        .field("Point emote", pointsdata.guild_emote, false)
                        .field("Active", pointsdata.active, false)
                        .field("Total points scored", pointsdata.total, false)
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
            }).await?;
        }
        None => {
            ctx.say("You need to chose a emote to use to collect points by using the `/points emote_set` command.").await?;
        }
    }

    Ok(())
}

/// Show the leaderboard for points on the server
///
/// Use this command to show the leaderboards for the points on this server.
#[poise::command(
    slash_command,
    required_permissions = "ADMINISTRATOR",
    category = "Points",
    guild_only,
    rename = "leaderboard"
)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let point_data = dbi::get_point_data(ctx.guild_id()).await?;
    let mut user_data = dbi::get_all_user_data(ctx.guild_id()).await?;

    // check if we have any point_data
    if let Some(points_data) = point_data {
        if user_data.len() == 0 {
            ctx.say("No points earned on this server yet.").await?;
        } else {
            // sort from most points to least points and slice into 20 entries per page
            user_data.sort_by_key(|a| std::cmp::Reverse(a.grammarpoints));
            let sliced_data: Vec<Vec<MyUser>> =
                user_data.chunks(20).map(|chunk| chunk.to_vec()).collect();

            let mut embeds: Vec<CreateEmbed> = Vec::with_capacity(sliced_data.len());
            for slice in sliced_data {
                let fields: Vec<(String, String, bool)> = slice
                    .iter()
                    .map(|user| {
                        (
                            format!("**{}**", user.discord_user.name),
                            format!("{} Points", user.grammarpoints),
                            true,
                        )
                    })
                    .collect();
                let embed = CreateEmbed::default()
                    .title(format!("Point Leaderboard for {}", ctx.guild_id().unwrap().name(&ctx).unwrap()))
                    .description(format!("Leaderboard for the points scored on this server. A total of **{} Points** have been scored on this server.", points_data.total))
                    .fields(fields)
                    .colour(Colour::BLUE)
                    .footer(|f| 
                        f.text(format!("Requsted by {}. Only they can change pages.", ctx.author().name))
                    ).to_owned();

                embeds.push(embed);
            }

            paginate_with_embeds(ctx, embeds).await?;
        }
    } else {
        ctx.say("No points earned on this server yet.").await?;
    }

    Ok(())
}
