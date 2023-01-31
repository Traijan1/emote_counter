use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};

use crate::database::DATABASE;

pub const COMMAND_STRING: &str = "count_emote_from";

pub async fn run(options: &[CommandDataOption]) -> String {
    let option = options
        .get(0)
        .expect("Expected String option")
        .resolved
        .as_ref()
        .expect("Expected String object");

    if let CommandDataOptionValue::String(value) = option {
        let current_date = chrono::offset::Utc::now().timestamp();
        let connection = DATABASE.lock().await;
        let mut sql = connection
            .prepare(&format!(
                "SELECT date FROM emotes WHERE emote_id LIKE '%{}%' AND date < ?",
                value
            ))
            .unwrap();
        let emotes: Vec<Result<i32, rusqlite::Error>> = sql
            .query_map([current_date], |row| Ok(row.get(0).unwrap()))
            .unwrap()
            .collect();

        println!("{}", value);

        format!("Count of {} is: {}", value, emotes.len())
    } else {
        String::new()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(COMMAND_STRING)
        .description("Get the count of all emotes from-to a certain date")
        .create_option(|option| {
            option
                .name("Start date")
                .description("The start date in format dd.mm.yyyy")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
