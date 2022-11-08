use std::collections::HashMap;
use std::hash::Hash;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

use crate::DATABASE;

pub async fn run(_: &[CommandDataOption]) -> String {
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

    for val in test {
        result += &format!("{} => {}\n", val.0, val.1);
    }

    result
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("count_all_emotes")
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
