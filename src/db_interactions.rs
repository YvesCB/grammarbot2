use log::{error, warn};
use surrealdb::engine::remote::ws::Client as SurrealClient;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use crate::types::*;

pub static DB: Surreal<SurrealClient> = Surreal::init();

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

    DB.use_ns("discordbot").use_db("grammarbot").await?;
    warn!("Using ns discordbot and db grammarbot");

    Ok(())
}

/// Some code for testing db interactions
#[allow(dead_code)]
pub async fn db_test() -> surrealdb::Result<()> {
    let del_tags: Result<Vec<Tag>, surrealdb::Error> = DB.delete("tag").await;
    match del_tags {
        Ok(tags) => println!("Deleted: {:?}", tags),
        Err(_) => println!("Table empty, nothing deleted."),
    }

    let created: Record = DB
        .create("tag")
        .content(Tag {
            name: "hello".to_string(),
            content: "Content of the hello tag!".to_string(),
        })
        .await?;
    println!("{:?}", created);

    let created: Record = DB
        .create("tag")
        .content(Tag {
            name: "goodbye".to_string(),
            content: "Content of the goodbye tag!".to_string(),
        })
        .await?;
    println!("{:?}", created);

    //now get all tags
    let tags: Vec<Tag> = DB.select("tag").await?;
    println!("{:?}", tags);

    let query = "DELETE from tag WHERE name == $tagname";
    let response = DB.query(query).bind(("tagname", "goodbye")).await?;
    println!("{:?}", response);

    let tags: Vec<Tag> = DB.select("tag").await?;
    println!("{:?}", tags);

    Ok(())
}

/// Create a tag entry in the tag table by passing its name and content
pub async fn create_tag(tag: &Tag) -> Result<(), DBIError> {
    let existing_tags: Vec<Tag> = DB.select("tag").await?;

    match existing_tags.iter().find(|t| t.name == tag.name) {
        Some(_) => return Err(DBIError::TagAlreadyExists),
        None => {
            let created: Record = DB.create("tag").content(tag).await?;
            warn!("{:?}", created);

            Ok(())
        }
    }
}

/// Get a tag by its name. Returns an `TagError::TagNotFound` if the tag doens't exist
pub async fn get_tag(tagname: &str) -> Result<Tag, DBIError> {
    let query = "SELECT * FROM tag WHERE name == $tagname";
    let mut response = DB.query(query).bind(("tagname", tagname)).await?;
    let tags: Vec<Tag> = response.take(0)?;

    // Note here that creation of tags prevents a name to be used multiple times.
    // Thus the resulting vector is either of length 0 or 1
    match tags.len() {
        0 => Err(DBIError::TagNotFound),
        _ => Ok(tags.first().unwrap().to_owned()),
    }
}

/// Returns a vector of all the tags in the DB. Can be of length 0
pub async fn get_all_tags() -> Result<Vec<Tag>, DBIError> {
    let tags: Vec<Tag> = DB.select("tag").await?;

    Ok(tags)
}

/// Removes a tag by its name. Returns `TagError::TagNotFound` if tag can't be found
pub async fn remove_tag(tagname: &str) -> Result<Tag, DBIError> {
    let mut response = DB
        .query("SELECT * FROM tag WHERE name == $tagname")
        .query("DELETE FROM tag WHERE name == $tagname")
        .bind(("tagname", tagname))
        .await?;
    let tags: Vec<Tag> = response.take(0)?;
    match tags.len() {
        0 => Err(DBIError::TagNotFound),
        _ => Ok(tags.first().unwrap().to_owned()),
    }
}

/// Get a vector of all the roles that users can asign to themselves. Can be of length 0.
pub async fn get_all_roles() -> Result<Vec<UserRole>, DBIError> {
    let roles: Vec<UserRole> = DB.select("role").await?;

    Ok(roles)
}

/// Add a role to the saved user-assignable roles. Returns `DBIError::RoleAlreadyExists` if the
/// role was already added previously
pub async fn add_role(role: UserRole) -> Result<(), DBIError> {
    let existing_roles: Vec<UserRole> = DB.select("role").await?;

    match existing_roles
        .iter()
        .find(|r| r.discordid == role.discordid)
    {
        Some(_) => return Err(DBIError::RoleAlreadyExists),
        None => {
            let created: Record = DB.create("role").content(role).await?;
            warn!("{:?}", created);

            Ok(())
        }
    }
}

/// Remove a role from the user-assignable roles. Returns `DBIError::RoleNotFound` if the role is
/// not in the database
pub async fn remove_role(role: UserRole) -> Result<UserRole, DBIError> {
    // We will add the error handling but in reality, this function will only be called with a role
    // that should exist.
    let mut response = DB
        .query("SELECT * FROM role WHERE discordid == $id")
        .query("DELETE FROM tag WHERE discordid == $id")
        .bind(("discordid", role.discordid))
        .await?;
    let roles: Vec<UserRole> = response.take(0)?;
    match roles.len() {
        0 => Err(DBIError::RoleNotFound),
        _ => Ok(roles.first().unwrap().to_owned()),
    }
}
