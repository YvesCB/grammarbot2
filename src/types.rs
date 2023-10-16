use poise::serenity_prelude::{Emoji, Message, Role, User};
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub name: String,
    pub content: String,
    pub creator: User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyUser {
    pub discord_id: String,
    pub discord_user: User,
    pub grammarpoints: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRole {
    pub guild_role: Role,
    pub emote: Emoji,
    pub desc: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoleMessage {
    pub messagetext: String,
    pub guild_message: Option<Message>,
    pub active: bool,
    pub message_by: User,
    pub posted_by: Option<User>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PointsData {
    pub guild_emote: Emoji,
    pub set_by: User,
    pub active: bool,
    pub total: u32,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    #[allow(dead_code)]
    id: Thing,
}

#[derive(Serialize, Deserialize)]
pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
pub enum DBIError {
    DBError(surrealdb::Error),
    TagAlreadyExists,
    TagNotFound,
    UserNotFound,
    RoleAlreadyExists,
    RoleNotFound,
    PointDataNotFound,
}

impl fmt::Display for DBIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DBIError::DBError(e) => write!(f, "Error when accessing the DB: {}", e),
            DBIError::TagAlreadyExists => write!(f, "Tag name already exists"),
            DBIError::TagNotFound => write!(f, "Tag name not found"),
            DBIError::UserNotFound => write!(f, "User not found"),
            DBIError::RoleAlreadyExists => write!(f, "Role already exists"),
            DBIError::RoleNotFound => write!(f, "Role not found"),
            DBIError::PointDataNotFound => write!(f, "Points data doesn't exist"),
        }
    }
}

impl error::Error for DBIError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DBIError::TagAlreadyExists => None,
            DBIError::TagNotFound => None,
            DBIError::UserNotFound => None,
            DBIError::RoleAlreadyExists => None,
            DBIError::RoleNotFound => None,
            DBIError::PointDataNotFound => None,
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            DBIError::DBError(ref e) => Some(e),
        }
    }
}

impl From<surrealdb::Error> for DBIError {
    fn from(err: surrealdb::Error) -> DBIError {
        DBIError::DBError(err)
    }
}

#[derive(Debug)]
pub struct LogError;

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to set up loggger")
    }
}
