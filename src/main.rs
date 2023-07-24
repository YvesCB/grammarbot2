use std::env;

fn main() {
    let client_id = env::var("DISCORD_CLIENTID").unwrap();
    let token = env::var("DISCORD_TOKEN").unwrap();

    println!("ID: {}, Token: {}", client_id, token);
}
