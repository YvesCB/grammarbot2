use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub tags: Vec<Tag>,
} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, crate::Data, Error>;
