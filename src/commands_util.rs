use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::Embed;

use crate::embed_tools::*;
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
    let commands = &ctx.framework().options().commands;
    dbg!(commands);
    Ok(())
}

/// Util function for creating pages that can be flipped through
#[poise::command(slash_command, category = "Various")]
pub async fn page_test(ctx: Context<'_>) -> Result<(), Error> {
    let texts = vec![
        "This is the first text".to_string(),
        "This is the second text".to_string(),
        "This is the last text".to_string(),
    ];

    let embeds = vec![
        CreateEmbed::default()
            .title("First one")
            .description("Some description for the first embed")
            .to_owned(),
        CreateEmbed::default()
            .title("Secon one")
            .description("Some description for the second one")
            .to_owned(),
    ];

    paginate_with_embeds(ctx, embeds).await?;

    Ok(())
}
