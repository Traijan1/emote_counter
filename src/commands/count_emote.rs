use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};

use crate::DATABASE;

pub async fn run(options: &[CommandDataOption]) -> String {
    let option = options
        .get(0)
        .expect("Expected String option")
        .resolved
        .as_ref()
        .expect("Expected String object");

    if let CommandDataOptionValue::String(value) = option {
        let count: i32 = match DATABASE.lock().await.query_row(
            &format!("SELECT count FROM emotes WHERE emote_id LIKE '%{}%'", value),
            [],
            |row| row.get(0),
        ) {
            Ok(value) => value,
            Err(err) => {
                println!("Error using count_emote: {}", err);
                0
            }
        };

        format!("Count of {} is: {}", value, count)
    } else {
        String::new()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("count_emote")
        .description("Get the count of an specified emote")
        .create_option(|option| {
            option
                .name("emote")
                .description("The emote you want the count of")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
