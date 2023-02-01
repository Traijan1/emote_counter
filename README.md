# Emote Counter

Let's count your server's emote usage, including those used as reactions.  
The database is `sqlite`.

## Commands

- `/count_emote [emote]`.
  - Get the count of a given emote
- `/count_all_emotes`.
  - Get a message listing all emotes.  
  The message will show 25 emotes. You can scroll through them using the ⬅️ and ➡️ reactions.

<br>

## Install

Prerequisite: You need to have [Rust](https://www.rust-lang.org/) installed.

After cloning the project:
- Create a file named `.env
- Write `BOT_TOKEN=your_discord_bot_token` into the file.
- Alternatively, if you only want to track emotes from your own server, write `SERVER_EMOTE_REGEX=regex_to_track_your_emotes`.  
For example `^TEST.*$` where `TEST` is the prefix of the emote. (This will match e.g. TEST_kappa)
- Run `cargo run --release`, the bot will now run
- You can also build the bot with `cargo build --release` and copy/move the output and .env file to another folder.