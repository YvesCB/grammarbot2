use log::warn;
use once_cell::sync::Lazy;
use poise::serenity_prelude::{Emoji, GuildId, Message, User};
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
    warn!(
        "Using ns {} and db {}",
        constants::DB_NS,
        constants::DB_DEFAULT_DB
    );

    Ok(())
}

async fn setdb(guildid: &Option<GuildId>) -> Result<(), DBIError> {
    let dbname = match guildid {
        Some(id) => id.0.to_string(),
        None => "global".to_string(),
    };
    DB.use_ns(constants::DB_NS).use_db(&dbname).await?;

    Ok(())
}

/// Create a tag in the database with the id equal to the tag name
pub async fn create_tag(tag: Tag, guildid: Option<GuildId>) -> Result<Tag, DBIError> {
    setdb(&guildid).await?;

    let existing_tag: Option<Tag> = DB.select((constants::DB_TAGS, &tag.name)).await?;

    match existing_tag {
        Some(_) => Err(DBIError::TagAlreadyExists),
        None => {
            let created_tag: Option<Tag> = DB
                .create((constants::DB_TAGS, &tag.name))
                .content(tag)
                .await?;
            Ok(created_tag.unwrap())
        }
    }
}

/// Get a tag by its name. Returns an `TagError::TagNotFound` if the tag doens't exist
pub async fn get_tag(tagname: &str, guildid: Option<GuildId>) -> Result<Tag, DBIError> {
    setdb(&guildid).await?;

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
    setdb(&guildid).await?;

    let tags: Vec<Tag> = DB.select(constants::DB_TAGS).await?;

    Ok(tags)
}

/// Removes a tag by its name. Returns `TagError::TagNotFound` if tag can't be found
pub async fn remove_tag(tagname: &str, guildid: Option<GuildId>) -> Result<Tag, DBIError> {
    setdb(&guildid).await?;

    let tag: Option<Tag> = DB.delete((constants::DB_TAGS, tagname)).await?;
    match tag {
        Some(t) => {
            warn!(
                "In {}, db_interaction::remove_tag: removed Tag: {:?}",
                &guildid.unwrap().0,
                &t
            );
            Ok(t)
        }
        None => Err(DBIError::TagNotFound),
    }
}

/// Get a vector of all the roles that users can asign to themselves. Can be of length 0.
pub async fn get_all_roles(guildid: Option<GuildId>) -> Result<Vec<UserRole>, DBIError> {
    setdb(&guildid).await?;

    let roles: Vec<UserRole> = DB.select(constants::DB_ROLES).await?;

    Ok(roles)
}

/// Add a role to the saved user-assignable roles. Returns `DBIError::RoleAlreadyExists` if the
/// role was already added previously
pub async fn add_role(role: UserRole, guildid: Option<GuildId>) -> Result<UserRole, DBIError> {
    setdb(&guildid).await?;

    let created: Option<UserRole> = DB
        .create((constants::DB_ROLES, role.guild_role.id.to_string()))
        .content(role)
        .await?;
    match created {
        Some(ur) => {
            warn!(
                "In {}, db_interaction::add_role: added UserRole: {:?}",
                &guildid.unwrap().0,
                &ur
            );
            Ok(ur)
        }
        None => Err(DBIError::RoleAlreadyExists),
    }
}

/// Get a role by its ID
pub async fn get_role(role_id: String, guildid: Option<GuildId>) -> Result<UserRole, DBIError> {
    setdb(&guildid).await?;

    let user_role: Option<UserRole> = DB.select((constants::DB_ROLES, role_id)).await?;

    match user_role {
        Some(ur) => Ok(ur),
        None => Err(DBIError::RoleNotFound),
    }
}

/// Remove a role from the user-assignable roles. Returns `DBIError::RoleNotFound` if the role is
/// not in the database
pub async fn remove_role(role: UserRole, guildid: Option<GuildId>) -> Result<UserRole, DBIError> {
    setdb(&guildid).await?;

    let removed_role: Option<UserRole> = DB
        .delete((constants::DB_ROLES, role.guild_role.id.to_string()))
        .await?;
    match removed_role {
        Some(ur) => {
            warn!(
                "In {}, db_interaction::remove_role: removed UserRole: {:?}",
                guildid.unwrap().0,
                &ur
            );
            Ok(ur)
        }
        None => Err(DBIError::RoleNotFound),
    }
}

/// Remove all roles from the db
pub async fn remove_all_roles(guildid: Option<GuildId>) -> Result<Vec<UserRole>, DBIError> {
    setdb(&guildid).await?;

    let roles: Vec<UserRole> = DB.delete(constants::DB_ROLES).await?;

    Ok(roles)
}

/// Returns the currently set role message. Returns None if no message is set
pub async fn get_role_message(guildid: Option<GuildId>) -> Result<Option<RoleMessage>, DBIError> {
    setdb(&guildid).await?;

    let cur_message: Option<RoleMessage> = DB.select((constants::DB_ROLEMSG, "0")).await?;
    Ok(cur_message)
}

/// Sets the current role message for the server. If one already exists, it is overwritten
pub async fn set_role_message(
    msg: String,
    user: &User,
    guildid: Option<GuildId>,
) -> Result<(), DBIError> {
    setdb(&guildid).await?;

    let cur_message: Option<RoleMessage> = DB.select((constants::DB_ROLEMSG, "0")).await?;
    match cur_message {
        Some(cur_msg) => {
            let _newmessage: Option<RoleMessage> = DB
                .update((constants::DB_ROLEMSG, "0"))
                .content(RoleMessage {
                    messagetext: msg.to_owned(),
                    message_by: user.to_owned(),
                    ..cur_msg
                })
                .await?;
            warn!(
                "In {}, db_interaction::set_role_message: changed role message from \"{}\" to \"{}\"",
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
                    guild_message: None,
                    active: false,
                    message_by: user.to_owned(),
                    posted_by: None,
                })
                .await?;
            warn!(
                "In {}, db_interactions::set_role_message: created role message \"{}\"",
                guildid.unwrap().0,
                &msg
            );
        }
    };

    Ok(())
}

/// Activate the role message by setting the id of the posted message and setting the active bool
/// to true
///
/// This function may only be called when a role message has been previously set!
pub async fn set_active_role_message(
    role_message: &RoleMessage,
    guild_message: Message,
    state: bool,
    user: &User,
    guildid: Option<GuildId>,
) -> Result<(), DBIError> {
    setdb(&guildid).await?;

    let _newmessage: Option<RoleMessage> = DB
        .update((constants::DB_ROLEMSG, "0"))
        .content(RoleMessage {
            guild_message: Some(guild_message),
            active: state,
            posted_by: Some(user.to_owned()),
            ..role_message.to_owned()
        })
        .await?;

    Ok(())
}

/// Get all user data
pub async fn get_all_user_data(guildid: Option<GuildId>) -> Result<Vec<MyUser>, DBIError> {
    setdb(&guildid).await?;

    let user_data: Vec<MyUser> = DB.select(constants::DB_USERS).await?;

    Ok(user_data)
}

/// Get the data for a specific user
pub async fn get_user_data(guildid: Option<GuildId>, user_id: u64) -> Result<MyUser, DBIError> {
    setdb(&guildid).await?;

    let user: Option<MyUser> = DB
        .select((constants::DB_USERS, user_id.to_string()))
        .await?;

    match user {
        Some(u) => Ok(u),
        None => Err(DBIError::UserNotFound),
    }
}

/// Get the current point emote record
pub async fn get_point_data(guildid: Option<GuildId>) -> Result<Option<PointsData>, DBIError> {
    setdb(&guildid).await?;

    let cur_point_emote: Option<PointsData> = DB.select((constants::DB_POINTEMOTE, "0")).await?;

    Ok(cur_point_emote)
}

/// Change points for a given user using the given function to apply to points
///
/// This will create a new user if no record exists in DB
pub async fn change_user_points(
    guildid: Option<GuildId>,
    user: User,
    func: fn(u32) -> u32,
) -> Result<MyUser, DBIError> {
    setdb(&guildid).await?;

    let cur_user: Option<MyUser> = DB
        .select((constants::DB_USERS, user.id.to_string()))
        .await?;

    let cur_point_stats: Option<PointsData> = DB.select((constants::DB_POINTEMOTE, "0")).await?;

    match (cur_user, cur_point_stats) {
        (Some(u), Some(p)) => {
            let new_user = MyUser {
                grammarpoints: func(u.grammarpoints),
                ..u
            };
            let new_points = PointsData {
                total: func(p.total),
                ..p
            };
            let _: Option<MyUser> = DB
                .update((constants::DB_USERS, user.id.to_string()))
                .content(new_user.to_owned())
                .await?;
            let _: Option<PointsData> = DB
                .update((constants::DB_POINTEMOTE, "0"))
                .content(new_points.to_owned())
                .await?;

            Ok(new_user)
        }
        (None, Some(p)) => {
            let new_points = PointsData {
                total: func(p.total),
                ..p
            };
            let new_user = MyUser {
                discord_id: user.id.to_string(),
                discord_user: user.to_owned(),
                grammarpoints: func(0),
            };
            let _: Option<MyUser> = DB
                .create((constants::DB_USERS, user.id.to_string()))
                .content(new_user.to_owned())
                .await?;
            let _: Option<PointsData> = DB
                .update((constants::DB_POINTEMOTE, "0"))
                .content(new_points.to_owned())
                .await?;

            Ok(new_user)
        }
        _ => Err(DBIError::PointDataNotFound),
    }
}

/// Updates the current point emote or will create the entry if none exists
pub async fn set_point_emote(
    point_emote: &Emoji,
    user: &User,
    guildid: Option<GuildId>,
) -> Result<(), DBIError> {
    setdb(&guildid).await?;

    let cur_points: Option<PointsData> = DB.select((constants::DB_POINTEMOTE, "0")).await?;
    match cur_points {
        Some(p) => {
            let _: Option<PointsData> = DB
                .update((constants::DB_POINTEMOTE, "0"))
                .content(PointsData {
                    guild_emote: point_emote.to_owned(),
                    set_by: user.to_owned(),
                    active: p.active,
                    total: p.total,
                })
                .await?;
            warn!(
                "In {}, db_interaction::set_point_emote: changed point emote from {} to {}",
                guildid.unwrap().0,
                &p.guild_emote.name,
                &point_emote.name
            );
        }
        None => {
            let _: Option<PointsData> = DB
                .create((constants::DB_POINTEMOTE, "0"))
                .content(PointsData {
                    guild_emote: point_emote.to_owned(),
                    set_by: user.to_owned(),
                    active: false,
                    total: 0,
                })
                .await?;
            warn!(
                "In {}, db_interactions::set_point_emote: created point emote \"{}\"",
                guildid.unwrap().0,
                &point_emote.name
            );
        }
    };

    Ok(())
}
