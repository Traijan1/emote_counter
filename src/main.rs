use std::collections::HashSet;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::prelude::*;

use lazy_static::lazy_static;
use rusqlite::Connection;

use dotenv::dotenv;
use std::env;

use log::{debug, error, info};

mod commands;

lazy_static! {
    static ref DATABASE: Mutex<Connection> =
        Mutex::new(Connection::open("database.db").expect("Can't open database.db"));
}

struct Handler;

#[command]
pub async fn hello(ctx: &Context, msg: &Message) -> CommandResult {
    let author = &msg.author.name;

    msg.channel_id
        .say(&ctx.http, format!("Hello {author}"))
        .await?;

    Ok(())
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "count_emote" => commands::count_emote::run(&command.data.options).await,
                "count_all_emotes" => commands::count_all_emotes::run(&command.data.options).await,
                "count_from_to" => commands::count_from_to::run(&command.data.options).await,
                _ => "This command does not exists".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                error!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot {
            let emotes = msg.guild_id.unwrap().emojis(&ctx.http).await.unwrap();

            for emote in emotes {
                if msg.content.contains(&emote.id.0.to_string()) {
                    let emote_string = &get_emote(emote.id.0.to_string(), emote.name.clone());
                    let count = msg.content.split(emote_string).into_iter().count() - 1;

                    for _ in 0..count {
                        add_emote_to_database(emote.id.0.to_string(), emote.name.clone()).await;
                    }
                }
            }
        }
    }

    async fn reaction_add(&self, _ctx: Context, add_reaction: serenity::model::prelude::Reaction) {
        let value = add_reaction.emoji.as_data();
        let data: Vec<&str> = value.split(":").collect();

        if data.len() == 2 {
            add_emote_to_database(data[1].to_string(), data[0].to_string()).await;
        } else {
            info!("{:?}", data);
        }
    }

    async fn reaction_remove(
        &self,
        _ctx: Context,
        removed_reaction: serenity::model::prelude::Reaction,
    ) {
        let value = removed_reaction.emoji.as_data();
        let data: Vec<&str> = value.split(":").collect();
        remove_emote_from_database(data[1].to_string(), data[0].to_string()).await;
    }

    async fn reaction_remove_all(
        &self,
        _ctx: Context,
        _channel_id: serenity::model::prelude::ChannelId,
        _removed_from_message_id: serenity::model::prelude::MessageId,
    ) {
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            commands::count_emote::register(command)
        })
        .await;

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            commands::count_all_emotes::register(command)
        })
        .await;

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            commands::count_from_to::register(command)
        })
        .await;
    }
}

#[group]
#[commands(hello)]
struct General;

#[tokio::main]
async fn main() {
    info!("Starting Bot");

    dotenv().ok();

    let _ = DATABASE.lock().await.execute(
        &format!("CREATE TABLE IF NOT EXISTS emotes (id INTEGER PRIMARY KEY AUTOINCREMENT, emote_id INTEGER, guild_id INTEGER, date INTEGER)"),
        [],
    ).unwrap();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("BOT_TOKEN").unwrap();

    debug!("Discord Token is: {}", token);

    let http = Http::new(&token);

    let (owners, _) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.owner.id)
        }
        Err(why) => panic!("Could not access application info {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("!"))
        .group(&GENERAL_GROUP);

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

fn get_emote(emote_id: String, name: String) -> String {
    format!("<:{}:{}>", name, emote_id)
}

async fn add_emote_to_database(emote_id: String, name: String) {
    let connection = DATABASE.lock().await;
    let mut sql = connection
        .prepare("INSERT INTO emotes (emote_id, guild_id, date) VALUES (?, ?, ?);")
        .unwrap();

    sql.execute([
        get_emote(emote_id, name),
        String::new(),
        chrono::offset::Utc::now().timestamp().to_string(),
    ])
    .unwrap();
}

async fn remove_emote_from_database(emote_id: String, name: String) {
    let connection = DATABASE.lock().await;
    let mut sql = connection
        .prepare(&format!(
            "SELECT id FROM emotes WHERE emote_id LIKE '%{}%'",
            get_emote(emote_id, name)
        ))
        .unwrap();

    let ids: Vec<Result<i32, rusqlite::Error>> = sql
        .query_map([], |row| Ok(row.get(0).unwrap()))
        .unwrap()
        .collect();

    // Delete latest or first emote?
    let emote_to_delete = ids[ids.len() - 1].as_ref().unwrap();

    sql = connection
        .prepare("DELETE FROM emotes WHERE id = ?")
        .unwrap();

    sql.execute([emote_to_delete]).unwrap();
}
