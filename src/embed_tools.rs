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
        (Some(texts), None) => texts.len() - 1,
        (None, Some(embeds)) => embeds.len() - 1,
        _ => 0,
    };

    let components = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(prev_button_id.clone())
            .style(serenity::ButtonStyle::Primary)
            .emoji('◀')
            .disabled(true),
        serenity::CreateButton::new(next_button_id.clone())
            .style(serenity::ButtonStyle::Primary)
            .emoji('▶')
            .disabled(page == last_page),
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

    let mut disable_prev;
    let mut disable_next;

    while let Some(interaction) = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .custom_ids(vec![prev_button_id.to_owned(), next_button_id.to_owned()])
        .timeout(Duration::from_secs(300))
        .next()
        .await
    {
        if interaction.data.custom_id == prev_button_id {
            page -= 1;
            disable_prev = page == 0;
            disable_next = false;
        } else {
            page += 1;
            disable_prev = false;
            disable_next = page == last_page;
        }
        let edit_components = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(prev_button_id.clone())
                .style(serenity::ButtonStyle::Primary)
                .emoji('◀')
                .disabled(disable_prev),
            serenity::CreateButton::new(next_button_id.clone())
                .style(serenity::ButtonStyle::Primary)
                .emoji('▶')
                .disabled(disable_next),
        ]);

        let interaction_response_message = match &texts_embeds {
            (Some(texts), None) => serenity::CreateInteractionResponseMessage::new()
                .components(vec![edit_components])
                .content(texts[page].clone()),
            (None, Some(embeds)) => serenity::CreateInteractionResponseMessage::new()
                .components(vec![edit_components])
                .embed(embeds[page].clone()),
            _ => serenity::CreateInteractionResponseMessage::new(),
        };

        let interaction_response =
            serenity::CreateInteractionResponse::UpdateMessage(interaction_response_message);

        interaction
            .create_response(ctx, interaction_response)
            .await?;
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
