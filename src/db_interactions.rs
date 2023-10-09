use log::{error, warn};
use once_cell::sync::Lazy;
use poise::serenity_prelude::GuildId;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use crate::constants;
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

    DB.use_ns(constants::DB_NS)
        .use_db(constants::DB_DEFAULT_DB)
        .await?;
    warn!("Using ns discordbot and db grammarbot");

    Ok(())
}

async fn setdb(guildid: Option<GuildId>) -> Result<(), DBIError> {
    let dbname = match guildid {
        Some(id) => id.0.to_string(),
        None => "global".to_string(),
    };
    DB.use_ns(constants::DB_NS).use_db(&dbname).await?;

    Ok(())
}

/// Create a tag in the database with the id equal to the tag name
pub async fn create_tag(tag: Tag, guildid: Option<GuildId>) -> Result<Tag, DBIError> {
    setdb(guildid).await?;

    let created_tag: Option<Tag> = DB
        .create((constants::DB_TAGS, &tag.name))
        .content(tag)
        .await?;

    match created_tag {
        Some(t) => {
            warn!("In: {} created Tag: {:?}", &guildid.unwrap().0, &t);
            Ok(t)
        }
        None => return Err(DBIError::TagAlreadyExists),
    }
}

/// Get a tag by its name. Returns an `TagError::TagNotFound` if the tag doens't exist
pub async fn get_tag(tagname: &str, guildid: Option<GuildId>) -> Result<Tag, DBIError> {
    setdb(guildid).await?;

    let tag: Option<Tag> = DB.select((constants::DB_TAGS, tagname)).await?;

    // Note here that creation of tags prevents a name to be used multiple times.
    // Thus the resulting vector is either of length 0 or 1
    match tag {
        Some(t) => Ok(t),
        None => Err(DBIError::TagNotFound),
    }
}

/// Returns a vector of all the tags in the DB. Can be of length 0
pub async fn get_all_tags(guildid: Option<GuildId>) -> Result<Vec<Tag>, DBIError> {
    setdb(guildid).await?;

    let tags: Vec<Tag> = DB.select(constants::DB_TAGS).await?;

    Ok(tags)
}

/// Removes a tag by its name. Returns `TagError::TagNotFound` if tag can't be found
pub async fn remove_tag(tagname: &str, guildid: Option<GuildId>) -> Result<Tag, DBIError> {
    setdb(guildid).await?;

    let tag: Option<Tag> = DB.delete((constants::DB_TAGS, tagname)).await?;
    match tag {
        Some(t) => {
            warn!("In: {} removed Tag: {:?}", &guildid.unwrap().0, &t);
            Ok(t)
        }
        None => Err(DBIError::TagNotFound),
    }
}

/// Get a vector of all the roles that users can asign to themselves. Can be of length 0.
pub async fn get_all_roles(guildid: Option<GuildId>) -> Result<Vec<UserRole>, DBIError> {
    setdb(guildid).await?;

    let roles: Vec<UserRole> = DB.select(constants::DB_ROLES).await?;

    Ok(roles)
}

/// Add a role to the saved user-assignable roles. Returns `DBIError::RoleAlreadyExists` if the
/// role was already added previously
pub async fn add_role(role: UserRole, guildid: Option<GuildId>) -> Result<UserRole, DBIError> {
    setdb(guildid).await?;

    let created: Option<UserRole> = DB
        .create((constants::DB_ROLES, role.guild_role.id.to_string()))
        .content(role)
        .await?;
    match created {
        Some(ur) => {
            warn!("In: {} added UserRole: {:?}", &guildid.unwrap().0, &ur);
            Ok(ur)
        }
        None => Err(DBIError::RoleAlreadyExists),
    }
}

/// Get a role by its ID
pub async fn get_role(role_id: String, guildid: Option<GuildId>) -> Result<UserRole, DBIError> {
    setdb(guildid).await?;

    let user_role: Option<UserRole> = DB.select((constants::DB_ROLES, role_id)).await?;

    match user_role {
        Some(ur) => Ok(ur),
        None => Err(DBIError::RoleNotFound),
    }
}

/// Remove a role from the user-assignable roles. Returns `DBIError::RoleNotFound` if the role is
/// not in the database
pub async fn remove_role(role: UserRole, guildid: Option<GuildId>) -> Result<UserRole, DBIError> {
    setdb(guildid).await?;

    let removed_role: Option<UserRole> = DB
        .delete((constants::DB_ROLES, role.guild_role.id.to_string()))
        .await?;
    match removed_role {
        Some(ur) => {
            warn!("In: {} removed UserRole: {:?}", guildid.unwrap().0, &ur);
            Ok(ur)
        }
        None => Err(DBIError::RoleNotFound),
    }
}

/// Returns the currently set role message. Returns None if no message is set
pub async fn get_role_message(guildid: Option<GuildId>) -> Result<Option<RoleMessage>, DBIError> {
    setdb(guildid).await?;

    let cur_message: Option<RoleMessage> = DB.select((constants::DB_ROLEMSG, "0")).await?;
    Ok(cur_message)
}

/// Sets the current role message for the server. If one already exists, it is overwritten
pub async fn set_role_message(msg: String, guildid: Option<GuildId>) -> Result<(), DBIError> {
    setdb(guildid).await?;

    let cur_message: Option<RoleMessage> = DB.select((constants::DB_ROLEMSG, "0")).await?;
    match cur_message {
        Some(cur_msg) => {
            let _newmessage: Option<RoleMessage> = DB
                .update((constants::DB_ROLEMSG, "0"))
                .content(RoleMessage {
                    messagetext: msg.to_owned(),
                    message_id: cur_msg.message_id.to_owned(),
                    active: cur_msg.active.to_owned(),
                })
                .await?;
            warn!(
                "In: {} changed role message from \"{}\" to \"{}\"",
                guildid.unwrap().0,
                &cur_msg.messagetext,
                &msg
            );
        }
        None => {
            let _newmessage: Option<RoleMessage> = DB
                .create((constants::DB_ROLEMSG, "0"))
                .content(RoleMessage {
                    messagetext: msg.to_owned(),
                    message_id: None,
                    active: false,
                })
                .await?;
            warn!(
                "In: {} created role message \"{}\"",
                guildid.unwrap().0,
                &msg
            );
        }
    };

    Ok(())
}
