use poise::serenity_prelude::Colour;
use poise::serenity_prelude::CreateEmbed;
use poise::Command;

use crate::embed_tools::*;
use crate::types::*;

/// Register and unregister commands
///
/// This command can be used to register and unregister commands of this bot.
/// Only the owner can use this command.
#[poise::command(
    slash_command,
    required_permissions = "ADMINISTRATOR",
    owners_only,
    category = "Admins",
    hide_in_help = true
)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    // use this for reference when creating buttons
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn autocomplete_commands<'a>(ctx: Context<'_>, partial: &'a str) -> Vec<String> {
    let commands = &ctx.framework().options().commands;

    let mut all_commands: Vec<String> = Vec::new();

    for command in commands.iter() {
        if command.subcommands.len() == 0 && !command.hide_in_help {
            all_commands.push(command.qualified_name.to_owned());
        } else {
            for subcom in command.subcommands.iter() {
                if !subcom.hide_in_help {
                    all_commands.push(subcom.qualified_name.to_owned());
                }
            }
        }
    }

    all_commands
        .iter()
        .filter(|c| c.contains(partial))
        .map(|res| res.to_owned())
        .collect::<Vec<String>>()
}

fn get_param_string(command: &Command<Data, Error>) -> String {
    let parameter_string: String = command
        .parameters
        .iter()
        .map(|param| {
            format!(
                "`{}{}`",
                param.name,
                match param.required {
                    true => String::from(""),
                    false => String::from("*"),
                }
            )
        })
        .collect::<Vec<String>>()
        .join(" ");

    parameter_string
}

fn get_detailed_param_string(command: &Command<Data, Error>) -> String {
    let parameter_string: String = command
        .parameters
        .iter()
        .map(|param| {
            format!(
                "`{}`: {}{}",
                param.name,
                match param.required {
                    true => String::from("(required) "),
                    false => String::from(" "),
                },
                match &param.description {
                    Some(desc) => desc.to_owned(),
                    None => String::from("-"),
                }
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    parameter_string
}

/// Print the help text
///
/// This prints the help text which lists all commands.
/// You can also specify an optional parameter to get info on a specific command.
#[poise::command(prefix_command, track_edits, slash_command, category = "Various")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "autocomplete_commands"]
    command: Option<String>,
) -> Result<(), Error> {
    let base_commands = &ctx.framework().options().commands;
    // Vec<Command<Data, Box<dyn Error + Send + Sync>>>
    let mut commands: Vec<&Command<Data, Error>> = Vec::with_capacity(base_commands.len());
    for base_command in base_commands {
        if !base_command.hide_in_help {
            commands.push(&base_command);
        }
    }

    // Cap 20 with the reasonalbe assumption that the bot is gonna have more than 10 and less than
    // 20 commands.
    let mut all_commands: Vec<&Command<Data, Error>> = Vec::with_capacity(20);

    for command in commands.iter() {
        if command.subcommands.len() == 0 && !command.hide_in_help {
            all_commands.push(*command);
        } else {
            for subcom in command.subcommands.iter() {
                if !subcom.hide_in_help {
                    all_commands.push(subcom);
                }
            }
        }
    }

    match command {
        Some(c) => {
            if let Some(bot_command) = all_commands
                .into_iter()
                .filter(|com| com.qualified_name == c)
                .next()
            {
                ctx.send(|b| {
                    b.embed(|e| {
                        e.title(format!(
                            "Help for: /{} {}",
                            &bot_command.qualified_name,
                            get_param_string(bot_command)
                        ))
                        .description(match bot_command.help_text {
                            Some(t) => t().replace("\n", " "),
                            None => String::from("-"),
                        })
                        .field("Parameters", get_detailed_param_string(bot_command), false)
                        .field("Guild only", format!("{}", &bot_command.guild_only), true)
                        .field("Required Perms", bot_command.required_permissions, true)
                        .colour(Colour::BLUE)
                        .footer(|f| {
                            f.text(format!(
                                "Optional parameters marked with *. Requsted by {}.",
                                ctx.author().name
                            ))
                        })
                    })
                })
                .await?;
            } else {
                ctx.say(format!("{} is not a recognized command.", c))
                    .await?;
            }
        }

        None => {
            // Cap 10 with the reasonable assumption that the bot will have no more than 10
            // categories of commands.
            let mut embeds: Vec<CreateEmbed> = Vec::with_capacity(10);
            for (idx, command) in commands.iter().enumerate() {
                let embed = CreateEmbed::default()
                    .title(format!(
                        "Help for: /{} {} (Page: {}/{})",
                        &command.qualified_name,
                        get_param_string(command),
                        idx + 1,
                        &commands.len()
                    ))
                    .description(match command.help_text {
                        Some(t) => t().replace("\n", " "),
                        None => String::from("-"),
                    })
                    .fields(
                        command
                            .subcommands
                            .iter()
                            .map(|com| {
                                (
                                    format!(
                                        "/{} {}",
                                        com.qualified_name.to_owned(),
                                        get_param_string(com)
                                    ),
                                    match &com.description {
                                        Some(d) => d.to_owned(),
                                        None => String::from("-"),
                                    },
                                    false,
                                )
                            })
                            .collect::<Vec<(String, String, bool)>>(),
                    )
                    .colour(Colour::BLUE)
                    .footer(|f| {
                        f.text(format!(
                            "Optional parameters marked with *. Requsted by {}.",
                            ctx.author().name
                        ))
                    })
                    .to_owned();

                embeds.push(embed);
            }

            paginate_with_embeds(ctx, embeds).await?;
        }
    }

    Ok(())
}
