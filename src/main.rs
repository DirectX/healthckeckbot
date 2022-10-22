use models::state::State;
use teloxide::{
    dispatching::dialogue::{
        serializer::{Json},
        ErasedStorage, SqliteStorage, Storage,
    },
    prelude::*,
};

use crate::models::state::Command;

type MyDialogue = Dialogue<State, ErasedStorage<State>>;
type MyStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

mod models;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    dotenv::dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();
    let storage: MyStorage = SqliteStorage::open("db.sqlite", Json).await.unwrap().erase();

    let handler = Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<State>, State>()
        .branch(dptree::case![State::Start].endpoint(start))
        .branch(
            dptree::case![State::GotNumber(n)]
                .branch(dptree::entry().filter_command::<Command>().endpoint(got_number))
                .branch(dptree::endpoint(invalid_command)),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    // Dispatcher::builder(
    //     bot,
    //     Update::filter_message()
    //         .enter_dialogue::<Message, InMemStorage<State>, State>()
    //         .branch(dptree::case![State::Start].endpoint(start))
    //         .branch(dptree::case![State::ReceiveFullName].endpoint(receive_full_name))
    //         .branch(dptree::case![State::ReceiveAge { full_name }].endpoint(receive_age))
    //         .branch(
    //             dptree::case![State::ReceiveLocation { full_name, age }].endpoint(receive_location),
    //         ),
    // )
    // .dependencies(dptree::deps![InMemStorage::<State>::new()])
    // .enable_ctrlc_handler()
    // .build()
    // .dispatch()
    // .await;
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(n)) => {
            dialogue.update(State::GotNumber(n)).await?;
            bot.send_message(
                msg.chat.id,
                format!("Remembered number {n}. Now use /get or /reset."),
            )
            .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please, send me a number.").await?;
        }
    }
    Ok(())
}

async fn got_number(
    bot: Bot,
    dialogue: MyDialogue,
    num: i32, // Available from `State::GotNumber`.
    msg: Message,
    cmd: Command,
) -> HandlerResult {
    match cmd {
        Command::Get => {
            bot.send_message(msg.chat.id, format!("Here is your number: {num}.")).await?;
        }
        Command::Reset => {
            dialogue.reset().await?;
            bot.send_message(msg.chat.id, "Number resetted.").await?;
        }
        Command::List => {
            bot.send_message(msg.chat.id, format!("List of events: {num}.")).await?;
        }
    }
    Ok(())
}

async fn invalid_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Please, send /list, /get or /reset.").await?;
    Ok(())
}

// async fn start(bot: Bot, dialogue: PersistentDialogue, msg: Message) -> HandlerResult {
//     bot.send_message(msg.chat.id, "Let's start! What's your full name?").await?;
//     dialogue.update(State::ReceiveFullName).await?;
//     Ok(())
// }

// async fn receive_full_name(bot: Bot, dialogue: PersistentDialogue, msg: Message) -> HandlerResult {
//     match msg.text() {
//         Some(text) => {
//             bot.send_message(msg.chat.id, "How old are you?").await?;
//             dialogue.update(State::ReceiveAge { full_name: text.into() }).await?;
//         }
//         None => {
//             bot.send_message(msg.chat.id, "Send me plain text.").await?;
//         }
//     }

//     Ok(())
// }

// async fn receive_age(
//     bot: Bot,
//     dialogue: PersistentDialogue,
//     full_name: String, // Available from `State::ReceiveAge`.
//     msg: Message,
// ) -> HandlerResult {
//     match msg.text().map(|text| text.parse::<u8>()) {
//         Some(Ok(age)) => {
//             bot.send_message(msg.chat.id, "What's your location?").await?;
//             dialogue.update(State::ReceiveLocation { full_name, age }).await?;
//         }
//         _ => {
//             bot.send_message(msg.chat.id, "Send me a number.").await?;
//         }
//     }

//     Ok(())
// }

// async fn receive_location(
//     bot: Bot,
//     dialogue: PersistentDialogue,
//     (full_name, age): (String, u8), // Available from `State::ReceiveLocation`.
//     msg: Message,
// ) -> HandlerResult {
//     match msg.text() {
//         Some(location) => {
//             let report = format!("Full name: {full_name}\nAge: {age}\nLocation: {location}");
//             bot.send_message(msg.chat.id, report).await?;
//             dialogue.exit().await?;
//         }
//         None => {
//             bot.send_message(msg.chat.id, "Send me plain text.").await?;
//         }
//     }

//     Ok(())
// }