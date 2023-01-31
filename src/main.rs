use std::collections::HashSet;
use std::env;

use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::prelude::*;

use dotenv::dotenv;
use log::info;

use crate::bot::Handler;
use crate::database::create_table;

mod bot;
mod commands;
mod database;

#[tokio::main]
async fn main() {
    info!("Starting Bot");

    dotenv().ok();
    create_table().await;

    let token = env::var("BOT_TOKEN").unwrap();
    let http = Http::new(&token);

    let (owners, _) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.owner.id)
        }
        Err(why) => panic!("Could not access application info {:?}", why),
    };

    let framework = StandardFramework::new().configure(|c| c.owners(owners).prefix("!"));

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating Client");

    if let Err(why) = client.start().await {
        panic!("Client error: {:?}", why);
    }
}
