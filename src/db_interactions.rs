use log::{error, warn};
use once_cell::sync::Lazy;
use poise::serenity_prelude::GuildId;
use surrealdb::engine::remote::ws::Client;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use crate::types::*;

pub static DB: Lazy<Surreal<Client>> = Lazy::new(Surreal::init);

/// Set up the connection to the surreal db server
pub async fn initiate_db() -> surrealdb::Result<()> {
    DB.connect::<Ws>("localhost:8000").await?;
    warn!("Connected to DB at localhost:8000");

    let dbuser = std::env::var("SURREAL_USER").expect("missing SURREAL_USER");
    let dbpass = std::env::var("SURREAL_PASS").expect("missing SURREAL_PASS");

    DB.signin(Root {
        username: &dbuser,
        password: &dbpass,
    })
    .await?;
    warn!("Signed into DB");

    DB.use_ns("discordbot").use_db("global").await?;
    warn!("Using ns discordbot and db grammarbot");

    Ok(())
}

/// Create a tag in the database with the id equal to the tag name
pub async fn create_tag(tag: Tag, guildid: Option<GuildId>) -> Result<Tag, DBIError> {
    let dbname = match guildid {
        Some(id) => id.0.to_string(),
        None => "global".to_string(),
    };
    DB.use_ns("discordbot").use_db(&dbname).await?;

    let created_tag: Option<Tag> = DB.create(("tag", &tag.name)).content(tag).await?;

    match created_tag {
        Some(t) => {
            warn!("In: {} created Tag: {:?}", &dbname, &t);
            Ok(t)
        }
        None => return Err(DBIError::TagAlreadyExists),
    }
}

/// Get a tag by its name. Returns an `TagError::TagNotFound` if the tag doens't exist
pub async fn get_tag(tagname: &str, guildid: Option<GuildId>) -> Result<Tag, DBIError> {
    let dbname = match guildid {
        Some(id) => id.0.to_string(),
        None => "global".to_string(),
    };
    DB.use_ns("discordbot").use_db(&dbname).await?;

    let tag: Option<Tag> = DB.select(("tag", tagname)).await?;

    // Note here that creation of tags prevents a name to be used multiple times.
    // Thus the resulting vector is either of length 0 or 1
    match tag {
        Some(t) => Ok(t),
        None => Err(DBIError::TagNotFound),
    }
}

/// Returns a vector of all the tags in the DB. Can be of length 0
pub async fn get_all_tags(guildid: Option<GuildId>) -> Result<Vec<Tag>, DBIError> {
    let dbname = match guildid {
        Some(id) => id.0.to_string(),
        None => "global".to_string(),
    };
    DB.use_ns("discordbot").use_db(&dbname).await?;

    let tags: Vec<Tag> = DB.select("tag").await?;

    Ok(tags)
}

/// Removes a tag by its name. Returns `TagError::TagNotFound` if tag can't be found
pub async fn remove_tag(tagname: &str, guildid: Option<GuildId>) -> Result<Tag, DBIError> {
    let dbname = match guildid {
        Some(id) => id.0.to_string(),
        None => "global".to_string(),
    };
    DB.use_ns("discordbot").use_db(&dbname).await?;

    let tag: Option<Tag> = DB.delete(("tag", tagname)).await?;
    match tag {
        Some(t) => {
            warn!("In: {} removed Tag: {:?}", &dbname, &t);
            Ok(t)
        }
        None => Err(DBIError::TagNotFound),
    }
}

/// Get a vector of all the roles that users can asign to themselves. Can be of length 0.
pub async fn get_all_roles() -> Result<Vec<UserRole>, DBIError> {
    let roles: Vec<UserRole> = DB.select("role").await?;

    Ok(roles)
}

/// Add a role to the saved user-assignable roles. Returns `DBIError::RoleAlreadyExists` if the
/// role was already added previously
pub async fn add_role(role: UserRole, guildid: Option<GuildId>) -> Result<UserRole, DBIError> {
    let dbname = match guildid {
        Some(id) => id.0.to_string(),
        None => "global".to_string(),
    };
    DB.use_ns("discordbot").use_db(&dbname).await?;

    let created: Option<UserRole> = DB
        .create(("role", role.guild_role.id.to_string()))
        .content(role)
        .await?;
    match created {
        Some(ur) => {
            warn!("In: {} added UserRole: {:?}", &dbname, &ur);
            Ok(ur)
        }
        None => Err(DBIError::RoleAlreadyExists),
    }
}

/// Remove a role from the user-assignable roles. Returns `DBIError::RoleNotFound` if the role is
/// not in the database
pub async fn remove_role(role: UserRole, guildid: Option<GuildId>) -> Result<UserRole, DBIError> {
    let dbname = match guildid {
        Some(id) => id.0.to_string(),
        None => "global".to_string(),
    };
    DB.use_ns("discordbot").use_db(&dbname).await?;

    let removed_role: Option<UserRole> =
        DB.delete(("role", role.guild_role.id.to_string())).await?;
    match removed_role {
        Some(ur) => {
            warn!("In: {} removed UserRole: {:?}", &dbname, &ur);
            Ok(ur)
        }
        None => Err(DBIError::RoleNotFound),
    }
}
