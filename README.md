# Complete GrammarBot rewrite
A project to finally rewrite GrammarBot. Switching everything over to slash commands and changing to another db (probably will end up being surrealdb).

The framework used for the rewrite will be [poise](https://github.com/serenity-rs/poise/), which is a framework for Discord's API. Poise is built on top of [serenity](https://github.com/serenity-rs/serenity)

## Capabilities
In its current state, the bot can:
* react to events such as messages, joins, guild changes, reactions etc.
* react to slash commands
* interact with a surrealdb instance in a basic way
* create tags, show tags and delete tags (which are basically pre-written messages)
* create embeds

## Roadmap
- [X] get basic bot running: Token, responding to a command/message
- [X] get basic slash commands
- [X] get db framework to work
    - [X] connect to db
    - [X] create db records
    - [X] read db records
- [X] logging
- [ ] rewrite commands

## Requirments
This bot relies on a running [SurrealDB](https://surrealdb.com/) server. The credentials for the server need to be in the env vars `SURREAL_USER` and `SURREAL_PASS`.
Further more the Token for the discord bot needs to be stored in the env var `DISCORD_TOKEN`

## Goal
The goal for this project is to have a bot that runs reliably with code that is easy to debug.
The code should also be easy to extend so that new features can added easily.
Discord has added a lot of new features to the API since the last time I wrote a bot in python. I want to try those features, such as slash commands and buttons etc.

Additionally, this will also just be a project that allows me to work more with Rust.
