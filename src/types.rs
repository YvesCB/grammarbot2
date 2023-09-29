use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyUser {
    pub name: String,
    pub userid: String,
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
pub enum TagError {
    DBError(surrealdb::Error),
    TagAlreadyExists,
    TagNotFound,
}

impl fmt::Display for TagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.clone() {
            TagError::DBError(e) => write!(f, "error when accessing the DB: {}", e),
            TagError::TagAlreadyExists => write!(f, "tag name already exists"),
            TagError::TagNotFound => write!(f, "tag name not found"),
        }
    }
}

impl error::Error for TagError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            TagError::TagAlreadyExists => None,
            TagError::TagNotFound => None,
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            TagError::DBError(ref e) => Some(e),
        }
    }
}

impl From<surrealdb::Error> for TagError {
    fn from(err: surrealdb::Error) -> TagError {
        TagError::DBError(err)
    }
}

#[derive(Debug)]
pub struct LogError;

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to set up loggger")
    }
}
