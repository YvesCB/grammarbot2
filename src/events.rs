use crate::serenity;
use poise::event::Event;

pub fn my_event_handler(ctx: &serenity::Context, event: &Event) {
    println!("Got event: {}", event.name());
    match event {
        Event::Message { new_message } => handle_message(&new_message),
        _ => {}
    }
}

fn handle_message(msg: &serenity::Message) {
    println!("Someone posted: {:?}", msg);
}
