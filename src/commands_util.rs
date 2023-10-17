use poise::serenity_prelude::{ButtonStyle, ReactionType};

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

/// A test for paging
#[poise::command(slash_command)]
pub async fn page_test(ctx: Context<'_>) -> Result<(), Error> {
    let mut page = 1;

    let reply = ctx
        .send(|m| {
            m.content(&page.to_string()).components(|c| {
                c.create_action_row(|r| {
                    r.create_button(|b| {
                        b.custom_id("page.prev")
                            .style(ButtonStyle::Primary)
                            .disabled(match page {
                                1 => true,
                                _ => false,
                            })
                            .emoji(ReactionType::Unicode("⬅️".to_string()))
                    })
                    .create_button(|b| {
                        b.custom_id("page.next")
                            .style(ButtonStyle::Primary)
                            .disabled(match page {
                                3 => true,
                                _ => false,
                            })
                            .emoji(ReactionType::Unicode("➡️".to_string()))
                    })
                })
            })
        })
        .await?;

    while let Some(interaction) = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .await
    {
        let pressed_button_id = &interaction.data.custom_id;

        match &**pressed_button_id {
            "page.prev" => {
                page -= 1;
                reply.edit(ctx, |b| b.content(&page.to_string())).await?;
            }
            "page.next" => {
                page += 1;
                reply.edit(ctx, |b| b.content(&page.to_string())).await?;
            }
            _ => {}
        };
    }

    reply.edit(ctx, |b| b.components(|f| f).content("")).await?;

    Ok(())
}
