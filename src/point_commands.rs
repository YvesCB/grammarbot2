use crate::dbi;
use crate::types::*;
use poise::serenity_prelude::Emoji;

/// Grammarpoint parent command
#[poise::command(slash_command, subcommands("emote_set", "emote_stats"))]
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
                })
            }).await?;
        }
        None => {
            ctx.say("You need to chose a emote to use to collect points by using the `/points emote_set` command.").await?;
        }
    }

    Ok(())
}
