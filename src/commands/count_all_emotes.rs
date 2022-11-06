use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

use crate::DATABASE;

pub async fn run(_: &[CommandDataOption]) -> String {
    let mutex = DATABASE.lock().await;
    let mut stmt = mutex.prepare("SELECT count, emote_id FROM emotes").unwrap();
    let value = stmt
        .query_map([], |row| {
            let first: String = row.get(1).unwrap();
            let second: usize = row.get(0).unwrap();

            Ok((second, first))
        })
        .unwrap();

    let mut result = String::new();
    let mut vec = vec![];

    for val in value {
        vec.push(val.unwrap());
    }

    vec.sort_by_key(|f| f.0);
    vec.reverse();

    for val in vec {
        result += &format!("{} => {}\n", val.1, val.0);
    }

    result
}

// format!("{} => {}\n", first, second)

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("count_all_emotes")
        .description("Get the count of all emotes")
}
