use std::collections::HashMap;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

use crate::database::DATABASE;

pub const COMMAND_STRING: &str = "count_all_emotes";
const MAX_EMOTES_PER_PAGE: usize = 25;

pub async fn run(_: &[CommandDataOption]) -> String {
    get_emotes_from_database(0).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(COMMAND_STRING)
        .description("Get the count of all emotes")
}

fn count_all_entries(iterable: Vec<String>) -> HashMap<String, usize> {
    let mut map: HashMap<String, usize> = HashMap::new();

    for value in iterable {
        if map.contains_key(&value) {
            let count = map.remove(&value).unwrap();
            map.insert(value, count + 1);
        } else {
            map.insert(value, 1);
        }
    }

    map
}

pub async fn get_emotes_from_database(offset: usize) -> String {
    let mutex = DATABASE.lock().await;
    let mut stmt = mutex.prepare("SELECT emote_id FROM emotes").unwrap();
    let value = stmt
        .query_map([], |row| {
            let emote_id: String = row.get(0).unwrap();

            Ok(emote_id)
        })
        .unwrap();

    let mut result = String::new();
    let mut vec = vec![];

    for val in value {
        vec.push(val.unwrap());
    }

    let hashmap = count_all_entries(vec);

    let mut test: Vec<_> = hashmap.iter().collect();
    test.sort_by_key(|f| f.1);
    test.reverse();

    for (emote, count) in test
        .iter()
        .skip(offset * MAX_EMOTES_PER_PAGE)
        .take(MAX_EMOTES_PER_PAGE)
    {
        result += &format!("{} => {}\n", emote, count);
    }

    result
}
