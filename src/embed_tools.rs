use std::time::Duration;

use poise::serenity_prelude::{CollectComponentInteraction, CreateEmbed, InteractionResponseType};

use crate::types::*;

pub async fn paginate_with_embeds(ctx: Context<'_>, embeds: Vec<CreateEmbed>) -> Result<(), Error> {
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    let mut page = 0;
    let last_page = embeds.len() - 1;

    ctx.send(|m| {
        m.embed(|b| {
            *b = embeds[page].to_owned();
            b
        })
        .components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| b.custom_id(&prev_button_id).disabled(true).emoji('◀'))
                    .create_button(|b| {
                        b.custom_id(&next_button_id)
                            .disabled(page == last_page)
                            .emoji('▶')
                    })
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
            if page == last_page {
                disabled_next = true;
            }
        } else if interaction.data.custom_id == prev_button_id {
            page -= 1;
            if page == 0 {
                disabled_prev = true;
            }
        } else {
            continue;
        }

        interaction
            .create_interaction_response(ctx, |b| {
                b.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|b| {
                        b.embed(|b| {
                            *b = embeds[page].to_owned();
                            b
                        })
                        .components(|c| {
                            c.create_action_row(|r| {
                                r.create_button(|b| {
                                    b.custom_id(&prev_button_id).disabled(true).emoji('◀')
                                })
                                .create_button(|b| {
                                    b.custom_id(&next_button_id).disabled(false).emoji('▶')
                                })
                            })
                        })
                        .components(|c| {
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

#[allow(dead_code)]
pub async fn paginate_with_text(ctx: Context<'_>, texts: Vec<String>) -> Result<(), Error> {
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    let mut page = 0;
    let last_page = texts.len() - 1;

    ctx.send(|m| {
        m.content(&texts[page]).components(|c| {
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
            if page == last_page {
                disabled_next = true;
            }
        } else if interaction.data.custom_id == prev_button_id {
            page -= 1;
            if page == 0 {
                disabled_prev = true;
            }
        } else {
            continue;
        }

        interaction
            .create_interaction_response(ctx, |b| {
                b.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|b| {
                        b.content(&texts[page]).components(|c| {
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
