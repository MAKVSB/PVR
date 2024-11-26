//! TODO: implement a TCP/IP client that will connect to a server with a canvas
//! and draw pixels to it.
//!
//! If you can draw your login to the canvas by the end of the seminar, you'll be awarded 2 points
//! for Gryffindor.
//!
//! # Communication protocol
//! After you connect to the server on a given TCP/IP v4 address and port, the following should
//! happen:
//! 1) You send a "Join" message that identifies you. You have to choose a nickname.
//! The format of the message is as follows:
//! `{"Join": {"name": "<nickname>"}}`
//!
//! - You have to send the message within two seconds. If you don't, the server will disconnect you.
//! - If the nickname is not unique (someone else has the same nickname), you will be disconnected.
//! - The nickname cannot be longer than `15` (UTF-8) bytes.
//! - You will be assigned a random color by the server.
//!
//! 2) The server will respond with a welcome message that contains the dimensions of the canvas
//! and a secret token that you have to use to draw to the canvas. The format of the welcome message
//! is as follows:
//! `{"Welcome": {"token": "<token>", "width": <width>, "height": <height>}}`
//!
//! 3) After you read the welcome message, you can start drawing to the canvas by sending the
//! following message:
//! `{"Draw": {"row": <row>, "col": <col>, "token": "<token>"}}`
//!
//! - If you try to draw outside of the bounds of the canvas, you will be disconnected.
//!
//! In addition to drawing, you can also send the following message to the server to get the canvas
//! state: `"GetState"`
//! The server will respond with a hashmap that contains the pixel positions sent by connected
//! users. Positions are represented with an array `[row, col]`:
//! ```
//! {"CanvasState": {"user-a": [[9,47],[10,2]], "user-b": [[0, 1], [5, 8]]}}
//! ```
//!
//! # Spam protection
//! You must not send a Draw or GetState message more often than once every 500 milliseconds. If
//! you do, you will receive a strike. After accumulating three strikes, you will be disconnected.
//!
//! # Inactivity protection
//! You have to send a Draw or GetState command at least once every five seconds, otherwise you
//! will be disconnected.
//!
//! # Message encoding
//! Messages between the server and the client are exchanged in JSON.
//! Each message is prefixed with a 4 byte little-endian number that specifies the amount of bytes
//! of the serialized JSON payload.
//! The maximum length of the payload is 256, larger messages will not be accepted by the server.
//!
//! # Notes
//! - If you do anything wrong, the server will send you an error that you can read to figure out
//! what's wrong. The format of the error message is `{"Error": "<error>"}`.
//! - Use `#[derive(serde::Serialize, serde::Deserialize)]` to build the protocol messages, don't
//! build the JSON messages by hand from strings.
//! - Any time you disconnect from the server, your pixels will be removed from the canvas.
//!
//! Bonus point if you can crash the server :)
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Request {
    Join {
        name: String
    },
    Welcome {
        token: String,
        width: u32,
        height: u32,
    },
    Draw {
        row: i32,
        col: i32, 
        token: String
    }

}

fn send_message(stream: &mut TcpStream, message: &str) -> io::Result<()> {
    // Check if the length is within the allowed limit
    let payload_length = message.len();

    // Convert the length to a 4-byte little-endian array
    let length_prefix = (payload_length as u32).to_le_bytes();

    // Create a combined buffer with the length prefix followed by the JSON payload
    let mut buffer = Vec::with_capacity(4 + payload_length);
    buffer.extend_from_slice(&length_prefix);
    buffer.extend_from_slice(message.as_bytes());

    // Send the buffer in a single write call
    stream.write_all(&buffer)?;

    Ok(())
}

fn main() {
    match TcpStream::connect("4.tcp.eu.ngrok.io:13502") {
        Ok(mut stream) => {

            let request = Request::Join { name: "mak0065".to_string() };
            let message = serde_json::to_string(&request).unwrap();
            println!("{}", message);
            send_message(&mut stream, &message).unwrap();

            let mut len_buffer = [0; 4];
            let mut global_token = "".to_string();
            let mut len: u32 = 0;
            match stream.read_exact(&mut len_buffer) {
                Ok(data) => {
                    len = u32::from_le_bytes(len_buffer);
                },
                Err(data) => todo!(),
            }
            let mut buffer = [0; 79];
            match stream.read_exact(&mut buffer) {
                Ok(data) => {
                    let response = str::from_utf8(&buffer[0..len as usize]).expect("Failed to read response");
                    println!("{}", response);
                    let a: Request = serde_json::from_str(response).unwrap();
                    if let Request::Welcome { token, width, height } = a {
                        global_token = token;
                    }
                }
                Err(e) => {
                    println!("Failed to read from the server: {}", e);
                }
            }

            let a = vec![
                vec![1,0,0,0,1,0,0,0,1,0,0,0,1,0,1,0,0,1,0,0,1,0,0,0,1,0,1,0,0,1,1,0,1,1,1],
                vec![1,1,0,1,1,0,0,1,0,1,0,0,1,1,0,0,1,0,1,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0],
                vec![1,0,1,0,1,0,0,1,1,1,0,0,1,0,0,0,1,0,1,0,0,1,0,1,0,0,1,0,1,0,0,0,1,1,0],
                vec![1,0,0,0,1,0,1,0,0,0,1,0,1,1,0,0,1,0,1,0,0,1,0,1,0,0,1,0,1,0,0,0,1,0,0],
                vec![1,0,0,0,1,0,1,0,0,0,1,0,1,0,1,0,0,1,0,0,0,0,1,0,0,0,1,0,0,1,1,0,1,1,1]
            ];
            
            for i in 0..5 {
                for j in 0..35 {
                    if a[i][j] == 1 {
                        let request = Request::Draw { row: i as i32 + 10, col: j as i32 + 10, token: global_token.clone() };
                        let message = serde_json::to_string(&request).unwrap();
                        send_message(&mut stream, message.as_str()).unwrap();
                        std::thread::sleep(Duration::from_millis(1000));
                    }
                }
            }




        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
        }
    }
}