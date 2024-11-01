mod network_types;
use network_types::Command;

use enigo::{Axis, Direction, Enigo, Keyboard, Mouse, Settings};
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Bind the TCP listener to port 8080
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on port 8080");

    // Create a channel for sending commands to the Enigo handler
    let (tx, mut rx) = mpsc::unbounded_channel::<Command>();

    // Spawn a dedicated task to handle Enigo commands
    tokio::spawn(async move {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        while let Some(command) = rx.recv().await {
            let event = match command {
                Command::MouseMove { x, y, coord } => enigo.move_mouse(x, y, coord),
                Command::MouseClick { button } => enigo.button(button, Direction::Click),
                Command::MousePress { button } => enigo.button(button, Direction::Press),
                Command::MouseRelease { button } => enigo.button(button, Direction::Release),
                Command::KeyPress { key } => enigo.key(key, Direction::Press),
                Command::KeyRelease { key } => enigo.key(key, Direction::Release),
            };
            event.unwrap()
        }
    });

    loop {
        // Accept incoming connections
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        // Clone the sender to move into the task
        let tx_clone = tx.clone();

        // Spawn a new task for each connection
        tokio::spawn(async move {
            let reader: BufReader<tokio::net::TcpStream> = BufReader::new(socket);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                println!("Received: {}", line);
                // match serde_json::from_str::<Command>(&line) {
                //     Ok(command) => {
                //         // Send the command to the Enigo handler
                //         if let Err(e) = tx_clone.send(command) {
                //             eprintln!("Failed to send command to Enigo handler: {}", e);
                //         }
                //     },
                //     Err(e) => {
                //         eprintln!("Failed to parse command: {}", e);
                //     }
                // }
            }

            println!("Connection from {} closed", addr);
        });
    }
}
