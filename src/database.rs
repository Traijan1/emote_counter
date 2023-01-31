use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::Connection;
use serenity::prelude::Mutex;

lazy_static! {
    pub static ref DATABASE: Mutex<Connection> =
        Mutex::new(Connection::open("database.db").expect("Can't open database.db"));
}

load_dotenv::load_dotenv!();
const SERVER_EMOTE_REGEX: &str = std::env!("SERVER_EMOTE_REGEX");

pub async fn create_table() {
    let _ = DATABASE.lock().await.execute(
        "CREATE TABLE IF NOT EXISTS emotes (id INTEGER PRIMARY KEY AUTOINCREMENT, emote_id INTEGER, guild_id INTEGER, date INTEGER)",
        [],
    ).unwrap();
}

pub async fn add_emote_to_database(emote_id: u64, name: &str) {
    if !is_trackable_emote(name) {
        return;
    }

    let connection = DATABASE.lock().await;
    let mut sql = connection
        .prepare("INSERT INTO emotes (emote_id, guild_id, date) VALUES (?, ?, ?);")
        .unwrap();

    sql.execute([
        get_emote(emote_id, name.into()),
        String::new(),
        chrono::offset::Utc::now().timestamp().to_string(),
    ])
    .unwrap();
}

pub async fn remove_emote_from_database(emote_id: u64, name: &str) {
    if !is_trackable_emote(name) {
        return;
    }

    let connection = DATABASE.lock().await;
    let mut sql = connection
        .prepare("SELECT id FROM emotes WHERE emote_id LIKE ?")
        .unwrap();

    let ids: Vec<Result<i32, rusqlite::Error>> = sql
        .query_map([format!("%{}%", get_emote(emote_id, name.into()))], |row| {
            Ok(row.get(0).unwrap())
        })
        .unwrap()
        .collect();

    // Delete latest or first emote?
    let emote_to_delete = ids[ids.len() - 1].as_ref().unwrap();

    sql = connection
        .prepare("DELETE FROM emotes WHERE id = ?")
        .unwrap();

    sql.execute([emote_to_delete]).unwrap();
}

pub fn get_emote(emote_id: u64, name: String) -> String {
    format!("<:{}:{}>", name, emote_id)
}

fn is_trackable_emote(emote: &str) -> bool {
    let pattern = Regex::new(&format!("^{SERVER_EMOTE_REGEX}.*$")).unwrap();
    pattern.is_match(emote)
}
