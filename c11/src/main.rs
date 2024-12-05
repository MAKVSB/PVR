//! TODO: implement a TCP/IP client that will connect to a server so that you can play a game
//!
//! You can send numbers that represent various *actions* to the server.
//! Actions control your *blob*. Use the provided code to
//! control the blob with your keyboard and shoot other players!
//! Be wary that the server is a bit moody, and it will periodically remap which numbers correspond
//! to which actions. You need to take this into account, otherwise the server will punish you.
//!
//! # Communication protocol
//! After you connect to the server on a given TCP/IP v4 address and port, the following should
//! happen:
//! 1) You send a [`ClientToServerMsg::Join`] message that identifies you.
//! You have to choose a nickname.
//!
//! - You have to send the message within two seconds. If you don't, the server will disconnect you.
//! - If the nickname is not unique (someone else has the same nickname), you will be disconnected.
//! - The nickname cannot be longer than `15` (UTF-8) bytes.
//!
//! 2) The server responds with a [`ServerToClientMsg::ActionMappingUpdate`] message, which maps
//! numbers to actions. The first message will always have the following mapping:
//! ```
//! 0 => MoveForward
//! 1 => MoveBackward
//! 2 => TurnLeft
//! 3 => TurnRight
//! 4 => Shield
//! 5 => Fire
//! 6..=10 => Invalid
//! ```
//!
//! Periodically, the server will decide to change the mapping of numbers to actions, and send you
//! the `ActionMappingUpdate` message again. You should read it and update your local mapping, so
//! that you send the correct actions to the server.
//!
//! Use either [`tokio::select`] or Tokio tasks to make sure that your code can concurrently handle
//! incoming server messages, events from the user's keyboard, and sending a heartbeat (see below).
//!
//! 3) Read key events from the keyboard using the provided code, and map some keyboard keys
//! to actions (and then actions to numbers). After an action is produced by the corresponding key,
//! send the [`ClientToServerMsg::PerformAction`] message to the server.
//!
//! If you send an invalid action, the server will freeze your blob for a few seconds, and increase
//! incoming damage by 100%.
//!
//! # Spam protection
//! You must not send a message more often than once every 0.1 milliseconds. If
//! you do, you will receive a strike. After accumulating three strikes, you will be disconnected.
//!
//! # Inactivity protection
//! You have to send the [`ClientToServerMsg::Heartbeat`] message at least once every five seconds,
//! otherwise you will be disconnected. You must not send it more often than once per second, though.
//!
//! # Message encoding
//! You can use the provided [`MessageReader`] and [`MessageWriter`] structs to communicate with the
//! server.
//!
//! Bonus point if you can crash the server :)

use crate::messages::{Action, ClientToServerMsg, ServerToClientMsg};
use crate::reader::MessageReader;
use crate::writer::MessageWriter;
use anyhow::anyhow;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use tokio::sync::Mutex;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::time;

mod messages;
mod reader;
mod writer;

/// You can use this macro for a bit nicer debugging output.
macro_rules! output {
    ($lit: literal) => {
        output!($lit,);
    };
    ($lit: literal, $($arg:tt),*) => {
        ::crossterm::terminal::disable_raw_mode().unwrap();
        println!($lit, $($arg),*);
        std::io::stdout().flush().unwrap();
        ::crossterm::terminal::enable_raw_mode().unwrap();
    };
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // Enable raw mode so that input key events are not buffered
    crossterm::terminal::enable_raw_mode()?;
    let result = run().await;
    crossterm::terminal::disable_raw_mode()?;
    result
}

async fn parse_received_message(data: ServerToClientMsg, mapping: Arc<Mutex<Vec<u8>>>) {
    match data {
        ServerToClientMsg::ActionMappingUpdate(mapping_data) => {
            let mut guard = mapping.lock().await;
            
            for n in 0..10 {
                match mapping_data[n] {
                    Action::TurnLeft => guard[0] = n as u8,
                    Action::TurnRight => guard[1] = n as u8,
                    Action::MoveForward => guard[3] = n as u8,
                    Action::MoveBackward => guard[4] = n as u8,
                    Action::Shield => guard[5] = n as u8,
                    Action::Fire => guard[6] = n as u8,
                    Action::Invalid => guard[7] = n as u8,
                }
            }
        },
        ServerToClientMsg::Error(_) => {
            panic!("failed to parse message");
        },
    }
}

async fn run() -> anyhow::Result<()> {
    // Connect to the server
    let client = TcpStream::connect(("7.tcp.eu.ngrok.io", 15305)).await?;
    let (stream, sink) = client.into_split();

    let (mut rx, mut tx) = (
        MessageReader::<ServerToClientMsg, _>::new(stream),
        Arc::new(Mutex::new(MessageWriter::<ClientToServerMsg, _>::new(sink))),
    );

    // Send the Join message with a unique nickname
    let nickname = "Zakk"; // Ensure this is unique
    if nickname.len() > 15 {
        return Err(anyhow!("Nickname too long!"));
    }

    tx.lock().await.send(ClientToServerMsg::Join {
        name: nickname.to_string(),
    })
    .await?;

    // Set up shared state for action mapping
    let action_mapping = Arc::new(Mutex::new(vec![0u8; 10]));

    // Task for reading from server
    let action_mapping_clone = action_mapping.clone();
    let read_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await.transpose()? {
            parse_received_message(message, action_mapping_clone.clone()).await;
        }
        anyhow::Ok(())
    });

    // Task for sending heartbeat
    let tx_clone = tx.clone();
    let heartbeat_task = tokio::spawn(async move {
        loop {
            tx_clone.lock().await.send(ClientToServerMsg::Heartbeat).await.unwrap();
            time::sleep(Duration::from_secs(2)).await;
        }
    });

    println!("connected");

    // Task for processing user input and sending actions
    let action_mapping_clone = action_mapping.clone();
    let tx_clone2 = tx.clone();
    let input_task = tokio::spawn(async move {
        let mut keyboard = EventStream::new();

        while let Some(Ok(Event::Key(key))) = keyboard.next().await {
            let mapping = action_mapping_clone.lock().await;


            let action_number = match key.code {
                KeyCode::Up => mapping[3],    // MoveForward
                KeyCode::Down => mapping[4],  // MoveBackward
                KeyCode::Left => mapping[0],  // TurnLeft
                KeyCode::Right => mapping[1], // TurnRight
                KeyCode::Char('s') => mapping[5], // Shield
                KeyCode::Char('f') => mapping[6], // Fire
                // KeyCode::Esc => panic!("gracefull shutdown"),
                _ => continue,
            };

            tx_clone2.lock().await.send(ClientToServerMsg::PerformAction (action_number))
                .await?;
            time::sleep(Duration::from_millis(0)).await; // Prevent spamming
        }

        anyhow::Ok(())
    });

    // Wait for tasks to complete (will loop indefinitely unless an error occurs)
    tokio::try_join!(read_task, heartbeat_task, input_task)?;

    Ok(())
}


