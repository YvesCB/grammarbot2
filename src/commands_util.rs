use std::time::Duration;

use poise::serenity_prelude::{
    ButtonStyle, CollectComponentInteraction, Embed, InteractionResponseType, ReactionType,
};

use crate::types::*;

/// Register and unregister commands
///
/// This command can be used to register and unregister commands of this bot.
/// Only the owner can use this command.
#[poise::command(slash_command, owners_only, category = "Admins")]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    // use this for reference when creating buttons
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Print the help text
///
/// This prints the help text which lists all commands.
/// You can also specify an optional parameter to get info on a specific command.
#[poise::command(prefix_command, track_edits, slash_command, category = "Various")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type ?help command for more info on a command.
You can edit your message to the bot and the bot will edit its response.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Util function for creating pages that can be flipped through
pub async fn page_test(ctx: Context<'_>, embeds: Vec<Embed>) -> Result<(), Error> {
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    let mut page = 0;
    let mut cur_embed = embeds.get(0).unwrap();

    let reply = ctx
        .send(|m| {
            m.content(page.to_string()).components(|c| {
                c.create_action_row(|r| {
                    r.create_button(|b| b.custom_id(&prev_button_id).disabled(true).emoji('◀'))
                        .create_button(|b| b.custom_id(&next_button_id).disabled(false).emoji('▶'))
                })
            })
        })
        .await?;

    // Loop through incoming interactions with the navigation buttons
    while let Some(interaction) = CollectComponentInteraction::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout after 10 min
        .timeout(Duration::from_secs(600))
        .await
    {
        let mut disabled_next = false;
        let mut disabled_prev = false;
        if interaction.data.custom_id == next_button_id {
            page += 1;
            if page == 3 {
                disabled_next = true;
            }
        } else if interaction.data.custom_id == prev_button_id {
            page -= 1;
            if page == 1 {
                disabled_prev = true;
            }
        } else {
            continue;
        }

        interaction
            .create_interaction_response(ctx, |b| {
                b.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|b| {
                        b.content(page).components(|c| {
                            c.create_action_row(|r| {
                                r.create_button(|b| {
                                    b.custom_id(&prev_button_id)
                                        .disabled(disabled_prev)
                                        .emoji('◀')
                                })
                                .create_button(|b| {
                                    b.custom_id(&next_button_id)
                                        .disabled(disabled_next)
                                        .emoji('▶')
                                })
                            })
                        })
                    })
            })
            .await?;
    }

    Ok(())
}
