use std::time::Duration;

use crate::types::*;
use poise::serenity_prelude as serenity;

async fn paginage_generic(
    ctx: Context<'_>,
    texts_embeds: (Option<Vec<String>>, Option<Vec<serenity::CreateEmbed>>),
) -> Result<(), Error> {
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    let mut page = 0;
    let last_page = match &texts_embeds {
        (Some(texts), None) => texts.len() == page,
        (None, Some(embeds)) => embeds.len() == page,
        _ => false,
    };

    let components = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(prev_button_id.clone())
            .style(serenity::ButtonStyle::Primary)
            .emoji('◀')
            .disabled(true),
        serenity::CreateButton::new(next_button_id.clone())
            .style(serenity::ButtonStyle::Primary)
            .emoji('▶')
            .disabled(last_page.to_owned()),
    ]);
    let builder = match &texts_embeds {
        (Some(texts), None) => poise::CreateReply::default()
            .content(texts[page].to_owned())
            .components(vec![components]),
        (None, Some(embeds)) => poise::CreateReply::default()
            .embed(embeds[page].to_owned())
            .components(vec![components]),
        _ => poise::CreateReply::default(),
    };

    let reply = ctx.send(builder).await?;

    let interactions = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .custom_ids(vec![prev_button_id.to_owned(), next_button_id.to_owned()])
        .timeout(Duration::from_secs(300));

    if let Some(interaction) = interactions.await {
        if interaction.data.custom_id == prev_button_id {
            page -= 1;
            let disable_prev = page == 0;
            let edit_components = serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new(prev_button_id.clone())
                    .style(serenity::ButtonStyle::Primary)
                    .emoji('◀')
                    .disabled(disable_prev),
                serenity::CreateButton::new(next_button_id.clone())
                    .style(serenity::ButtonStyle::Primary)
                    .emoji('▶'),
            ]);
            let edit_builder = match &texts_embeds {
                (Some(texts), None) => poise::CreateReply::default()
                    .content(texts[page].clone())
                    .components(vec![edit_components]),
                (None, Some(embeds)) => poise::CreateReply::default()
                    .embed(embeds[page].clone())
                    .components(vec![edit_components]),
                _ => poise::CreateReply::default(),
            };
            reply.edit(ctx, edit_builder).await?;
        } else {
            page += 1;
            let last_page = match &texts_embeds {
                (Some(texts), None) => texts.len() == page,
                (None, Some(embeds)) => embeds.len() == page,
                _ => false,
            };
            let disable_next = last_page;
            let edit_components = serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new(prev_button_id.to_owned())
                    .style(serenity::ButtonStyle::Primary)
                    .emoji('◀'),
                serenity::CreateButton::new(next_button_id.to_owned())
                    .style(serenity::ButtonStyle::Primary)
                    .emoji('▶')
                    .disabled(disable_next),
            ]);
            let edit_builder = match &texts_embeds {
                (Some(texts), None) => poise::CreateReply::default()
                    .content(texts[page].clone())
                    .components(vec![edit_components]),
                (None, Some(embeds)) => poise::CreateReply::default()
                    .embed(embeds[page].clone())
                    .components(vec![edit_components]),
                _ => poise::CreateReply::default(),
            };
            reply.edit(ctx, edit_builder).await?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn paginate_with_embeds(
    ctx: Context<'_>,
    embeds: Vec<serenity::CreateEmbed>,
) -> Result<(), Error> {
    // Define some unique identifiers for the navigation buttons
    paginage_generic(ctx, (None, Some(embeds))).await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn paginate_with_text(ctx: Context<'_>, texts: Vec<String>) -> Result<(), Error> {
    paginage_generic(ctx, (Some(texts), None)).await?;

    Ok(())
}
