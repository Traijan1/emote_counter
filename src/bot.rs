use std::collections::HashMap;

use commands::count_all_emotes::get_emotes_from_database;
use log::{error, info};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::{MessageId, ReactionType};
use serenity::prelude::*;

use crate::commands::{self, count_all_emotes, count_emote, count_from_to};
use crate::database::{add_emote_to_database, get_emote, remove_emote_from_database};

const LEFT_ARROW: &str = "⬅️";
const RIGHT_ARROW: &str = "➡️";

struct Paging;

impl TypeMapKey for Paging {
    type Value = HashMap<MessageId, usize>;
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot {
            let emotes = msg.guild_id.unwrap().emojis(&ctx.http).await.unwrap();

            for emote in emotes {
                if msg.content.contains(&emote.id.0.to_string()) {
                    let emote_string = &get_emote(emote.id.0, emote.name.clone());
                    let count = msg.content.split(emote_string).into_iter().count() - 1;

                    for _ in 0..count {
                        add_emote_to_database(emote.id.0, &emote.name).await;
                    }
                }
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: serenity::model::prelude::Reaction) {
        if add_reaction.emoji.unicode_eq(LEFT_ARROW) || add_reaction.emoji.unicode_eq(RIGHT_ARROW) {
            if add_reaction.user(&ctx.http).await.unwrap().bot {
                return;
            }

            // Get page state for message id

            let mut data = ctx.data.write().await;
            let map = data.get_mut::<Paging>().unwrap();

            let page = map.get_mut(&add_reaction.message_id).unwrap();

            // Set new state (value -1 or +1)

            if add_reaction.emoji.unicode_eq(LEFT_ARROW) {
                if *page == 0 {
                    add_reaction.delete(&ctx.http).await.unwrap();
                    return;
                }

                *page -= 1;
            } else if add_reaction.emoji.unicode_eq(RIGHT_ARROW) {
                *page += 1;
            }

            let mesage_value = &get_emotes_from_database(*page).await;

            if mesage_value.is_empty() {
                add_reaction.delete(&ctx.http).await.unwrap();
                *page -= 1;
                return;
            }

            // Edit message with new content (next or previous "page")
            add_reaction
                .message(&ctx.http)
                .await
                .unwrap()
                .edit(&ctx.http, |message| message.content(mesage_value))
                .await
                .unwrap();

            add_reaction.delete(&ctx.http).await.unwrap();
        }

        let value = add_reaction.emoji.as_data();
        let data: Vec<&str> = value.split(':').collect();

        if data.len() == 2 {
            let id = data[1].parse().unwrap();
            add_emote_to_database(id, data[0]).await;
        } else {
            info!("{:?}", data);
        }
    }

    async fn reaction_remove(
        &self,
        _ctx: Context,
        removed_reaction: serenity::model::prelude::Reaction,
    ) {
        if removed_reaction.emoji.unicode_eq(LEFT_ARROW)
            || removed_reaction.emoji.unicode_eq(RIGHT_ARROW)
        {
            return;
        }

        let value = removed_reaction.emoji.as_data();
        let data: Vec<&str> = value.split(':').collect();

        let id = data[1].parse().unwrap();
        remove_emote_from_database(id, data[0]).await;
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        ctx.data.write().await.insert::<Paging>(HashMap::new());

        Command::create_global_application_command(&ctx.http, |command| {
            commands::count_emote::register(command)
        })
        .await
        .unwrap();

        Command::create_global_application_command(&ctx.http, |command| {
            commands::count_all_emotes::register(command)
        })
        .await
        .unwrap();

        // Command::create_global_application_command(&ctx.http, |command| {
        //     commands::count_from_to::register(command)
        // })
        // .await
        // .unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let mut used_all_emote_command = false;

            let content = match command.data.name.as_str() {
                count_emote::COMMAND_STRING => {
                    commands::count_emote::run(&command.data.options).await
                }
                count_all_emotes::COMMAND_STRING => {
                    used_all_emote_command = true;
                    commands::count_all_emotes::run(&command.data.options).await
                }
                // count_from_to::COMMAND_STRING => {
                //     commands::count_from_to::run(&command.data.options).await
                // }
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

            if used_all_emote_command {
                let message = command.get_interaction_response(&ctx.http).await;

                if let Ok(message) = message {
                    let mut data = ctx.data.write().await;
                    let map = data.get_mut::<Paging>().unwrap();
                    let id = &message.id;

                    if !map.contains_key(id) {
                        map.insert(*id, 0);
                    }

                    message
                        .react(&ctx.http, ReactionType::Unicode(LEFT_ARROW.into()))
                        .await
                        .unwrap();
                    message
                        .react(&ctx.http, ReactionType::Unicode(RIGHT_ARROW.into()))
                        .await
                        .unwrap();
                }
            }
        }
    }
}
