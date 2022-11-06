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
                    let emote_string = &format!("<:{}:{}>", emote.name, emote.id.0);
                    let count = msg.content.split(emote_string).into_iter().count() - 1;

                    add_emote_count(emote.id.0.to_string(), emote.name, count as i32).await;
                }
            }
        }
    }

    async fn reaction_add(&self, _ctx: Context, add_reaction: serenity::model::prelude::Reaction) {
        let value = add_reaction.emoji.as_data();
        let data: Vec<&str> = value.split(":").collect();

        if data.len() == 2 {
            add_emote_count(data[1].to_string(), data[0].to_string(), 1).await;
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
        add_emote_count(data[1].to_string(), data[0].to_string(), -1).await;
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
        &format!("CREATE TABLE IF NOT EXISTS emotes (id INTEGER PRIMARY KEY AUTOINCREMENT, emote_id INTEGER, count INTEGER, guild_id INTEGER)"),
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

async fn add_emote_count(emote_id: String, name: String, count: i32) {
    let sql = &format!(
        "SELECT count from emotes where emote_id LIKE '%{}%'",
        emote_id
    );

    let mut table_count: i32 = match DATABASE.lock().await.query_row(sql, [], |row| row.get(0)) {
        Ok(value) => value,
        Err(err) => {
            println!("Error: {}", err);
            -1
        }
    };

    if table_count == -1 {
        let _ = DATABASE
            .lock()
            .await
            .execute(
                &format!(
                    "INSERT INTO emotes (emote_id, count, guild_id) VALUES ('{}', {}, {})",
                    format!("<:{}:{}>", name, emote_id),
                    if count > 0 { count } else { 0 },
                    "597433199567568896"
                ),
                [],
            )
            .unwrap();

        println!("{}", if count > 0 { count } else { 0 });
    } else {
        table_count += count as i32;

        if table_count > 0 {
            DATABASE
                .lock()
                .await
                .execute(
                    &format!(
                        "UPDATE emotes set count = {} where emote_id LIKE '%{}%'",
                        table_count, emote_id
                    ),
                    [],
                )
                .unwrap();
        }
    }
}
