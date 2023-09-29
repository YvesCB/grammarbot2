use surrealdb::engine::remote::ws::Client as SurrealClient;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use crate::types::*;

pub static DB: Surreal<SurrealClient> = Surreal::init();

/// Set up the connection to the surreal db server
pub async fn initiate_db() -> surrealdb::Result<()> {
    DB.connect::<Ws>("localhost:8000").await?;
    println!("Connected to DB at localhost:8000");

    let dbuser = std::env::var("SURREAL_USER").expect("missing SURREAL_USER");
    let dbpass = std::env::var("SURREAL_PASS").expect("missing SURREAL_PASS");

    DB.signin(Root {
        username: &dbuser,
        password: &dbpass,
    })
    .await?;
    println!("Signed into DB");

    DB.use_ns("discordbot").use_db("grammarbot").await?;
    println!("Using ns discordbot and db grammarbot");

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
    let del_users: Result<Vec<MyUser>, surrealdb::Error> = DB.delete("user").await;
    match del_users {
        Ok(users) => println!("Deleted: {:?}", users),
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

    // and the user
    let users: Vec<MyUser> = DB.select("user").await?;
    println!("{:?}", users);

    Ok(())
}

/// Create a tag entry in the tag table by passing its name and content
pub async fn create_tag(tag: &Tag) -> Result<(), TagError> {
    let existing_tags: Vec<Tag> = DB.select("tag").await?;

    match existing_tags.iter().find(|t| t.name == tag.name) {
        Some(_) => return Err(TagError::TagAlreadyExists),
        None => {
            let created: Record = DB.create("tag").content(tag).await?;
            println!("{:?}", created);

            Ok(())
        }
    }
}

/// Get a tag by its name. Returns an `TagError::TagNotFound` if the tag doens't exist
pub async fn get_tag(tagname: &str) -> Result<Tag, TagError> {
    let query = "SELECT * FROM tag WHERE name == $tagname";
    let mut response = DB.query(query).bind(("tagname", tagname)).await?;
    let tags: Vec<Tag> = response.take(0)?;

    // Note here that creation of tags prevents a name to be used multiple times.
    // Thus the resulting vector is either of length 0 or 1
    match tags.len() {
        0 => Err(TagError::TagNotFound),
        _ => Ok(tags.first().unwrap().to_owned()),
    }
}

/// Returns a vector of all the tags in the DB. Can be of length 0
pub async fn get_all_tags() -> Result<Vec<Tag>, TagError> {
    let tags: Vec<Tag> = DB.select("tag").await?;

    Ok(tags)
}

/// Removes a tag by its name. Returns `TagError::TagNotFound` if tag can't be found
pub async fn remove_tag(tagname: &str) -> Result<Tag, TagError> {
    let mut response = DB
        .query("SELECT * FROM tag WHERE name == $tagname")
        .query("DELETE FROM tag WHERE name == $tagname")
        .bind(("tagname", tagname))
        .await?;
    let tags: Vec<Tag> = response.take(0)?;
    match tags.len() {
        0 => Err(TagError::TagNotFound),
        _ => Ok(tags.first().unwrap().to_owned()),
    }
}
