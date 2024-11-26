//! TODO: implement a TCP/IP client that will connect to a server and figure out a password to
//! open a VAULT.
//!
//! If you can guess the password to the vault by the end of the seminar, you'll be awarded 2 points
//! for Gryffindor.
//!
//! # Communication protocol
//! After you connect to the server on a given TCP/IP v4 address and port, the following should
//! happen:
//! 1) You send a string that identifies you. You have to choose a nickname.
//!
//! - You have to send the message within two seconds. If you don't, the server will disconnect you.
//! - If the nickname is not unique (someone else has the same nickname), you will be disconnected.
//! - The nickname cannot be longer than `15` (UTF-8) bytes.
//!
//! 2) The following communication happens in a lockstep. You send a string that contains your guess
//! of the password. The server then responds either with:
//! - "correct" if you have guessed the password correctly
//! - "incorrect" if your password guess was wrong
//! - a string containing an error if some error has happened
//!
//! # Spam protection
//! - You must not send a message more often than once every 0.1 milliseconds. If
//!   you do, you will receive a strike. After accumulating three strikes, you will be disconnected.
//! - You must not make more than 10 thousand password guesses. After 10k attempts, you will be
//!   disconnected.
//!
//! # Inactivity protection
//! You have to send a message at least once every five seconds, otherwise you will be disconnected.
//!
//! # Message encoding
//! The encoding is similar to last week, although this time, each message is a simple UTF-8 string,
//! there is no JSON involved. You can use the provided `MessageReader` and `MessageWriter` structs
//! to communicate with the server.
//!
//! Bonus point if you can crash the server :)

use std::{net::TcpStream, time::{Duration, Instant}};

mod reader;
mod writer;

fn main() {
    match TcpStream::connect("4.tcp.eu.ngrok.io:13502") {
        Ok(mut stream) => {
            let mut writer = writer::MessageWriter::new(&stream);
            let mut reader = reader::MessageReader::new(&stream);

            writer.write("\u{FFFF}MAK0065(2)").unwrap();

            let mut correct_pass = String::from("\u{FFFF}");

            for j in 0..9 {
                let mut max: (char, f64) = ('a', 0.0);
                for c in  'a'..'z' {
                    let mut pass_try = correct_pass.clone();
                    pass_try.push(c);
                    let mut tries_vec = Vec::new();
                    for i in 0..2 {
                        let start = Instant::now();
                        writer.write(pass_try.as_str()).unwrap();
                        let response = reader.read().unwrap();
                        let duration = start.elapsed().as_millis();
                        if response.unwrap() == "correct" {
                            println!("Found: {:?}", correct_pass.as_str());
                        }
                        tries_vec.push(duration);
                    }
    
                    let sum: u128 = tries_vec.iter().sum();
                    let count = tries_vec.len();
                    let average = sum as f64 / count as f64;
    
                    if average > max.1 {
                        max = (c, average)
                    }
                }
                correct_pass.push(max.0);
                println!("{:?}", correct_pass.as_str());
            }

            // ferrisftw
            // let request = Request::Join { name: "mak0065".to_string() };
            // let message = "mak0065";
            // println!("{}", message);
            // send_message(&mut stream, &message).unwrap();

            // let mut len_buffer = [0; 4];
            // let mut global_token = "".to_string();
            // let mut len: u32 = 0;
            // match stream.read_exact(&mut len_buffer) {
            //     Ok(data) => {
            //         len = u32::from_le_bytes(len_buffer);
            //     },
            //     Err(data) => todo!(),
            // }
            // let mut buffer = [0; 79];
            // match stream.read_exact(&mut buffer) {
            //     Ok(data) => {
            //         let response = str::from_utf8(&buffer[0..len as usize]).expect("Failed to read response");
            //         println!("{}", response);
            //         let a: Request = serde_json::from_str(response).unwrap();
            //         if let Request::Welcome { token, width, height } = a {
            //             global_token = token;
            //         }
            //     }
            //     Err(e) => {
            //         println!("Failed to read from the server: {}", e);
            //     }
            // }


            // let mut x: i32 = 0;
            // let mut y: i32 = 0;
            // let mut dirx: i32 = 1;
            // let mut diry: i32 = 0;

            // loop {
            //     let request = Request::Draw { row: y + 50, col: x + 50, token: global_token.clone() };
            //     let message = serde_json::to_string(&request).unwrap();
            //     send_message(&mut stream, message.as_str()).unwrap();
            //     std::thread::sleep(Duration::from_millis(1000));

            //     x += dirx;
            //     y += diry;

            //     if x > 5 {
            //         dirx = 0;
            //         diry = 1;
            //     }
            //     if y > 5 {
            //         diry = 0;
            //         dirx = -1;
            //     }
            //     if y == 0 & di{
            //         diry = -1;
            //         dirx = 0;
            //     }

            // }

        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
        }
    }
}
