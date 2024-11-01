mod network_types;
use enigo::Coordinate;
use network_types::Command;

use rdev::{listen, EventType};
use serde::Serialize;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

#[derive(Serialize, Debug)]
#[serde(tag = "action")]
enum InputCommand {
    MouseMove { x: i32, y: i32 },
    MouseClick { button: String },
    KeyPress { key: String },
    KeyRelease { key: String },
}

fn map_mouse_buttons(button: rdev::Button) -> enigo::Button {
    match button {
        rdev::Button::Left => enigo::Button::Left,
        rdev::Button::Right => enigo::Button::Right,
        rdev::Button::Middle => enigo::Button::Middle,
        rdev::Button::Unknown(_) => panic!("Unknown key"),
    }
}

fn map_keyboard_keys(key: rdev::Key) -> enigo::Key {
    match key {
        rdev::Key::Alt => enigo::Key::Alt,
        rdev::Key::AltGr => enigo::Key::Alt, // HUH ?
        rdev::Key::Backspace => enigo::Key::Backspace,
        rdev::Key::CapsLock => enigo::Key::CapsLock,
        rdev::Key::ControlLeft => enigo::Key::Control,
        rdev::Key::ControlRight => todo!(),
        rdev::Key::Delete => enigo::Key::Delete,
        rdev::Key::DownArrow => enigo::Key::DownArrow,
        rdev::Key::End => enigo::Key::End,
        rdev::Key::Escape => enigo::Key::Escape,
        rdev::Key::F1 => enigo::Key::F1,
        rdev::Key::F10 => enigo::Key::F10,
        rdev::Key::F11 => enigo::Key::F11,
        rdev::Key::F12 => enigo::Key::F12,
        rdev::Key::F2 => enigo::Key::F2,
        rdev::Key::F3 => enigo::Key::F3,
        rdev::Key::F4 => enigo::Key::F4,
        rdev::Key::F5 => enigo::Key::F5,
        rdev::Key::F6 => enigo::Key::F6,
        rdev::Key::F7 => enigo::Key::F7,
        rdev::Key::F8 => enigo::Key::F8,
        rdev::Key::F9 => enigo::Key::F9,
        rdev::Key::Home => enigo::Key::Home,
        rdev::Key::LeftArrow => enigo::Key::LeftArrow,
        rdev::Key::MetaLeft => enigo::Key::Meta,
        rdev::Key::MetaRight => enigo::Key::Meta,
        rdev::Key::PageDown => enigo::Key::PageDown,
        rdev::Key::PageUp => enigo::Key::PageUp,
        rdev::Key::Return => enigo::Key::Return,
        rdev::Key::RightArrow => enigo::Key::RightArrow,
        rdev::Key::ShiftLeft => enigo::Key::Shift,
        rdev::Key::ShiftRight => enigo::Key::Shift, // HUH ?
        rdev::Key::Space => enigo::Key::Space,
        rdev::Key::Tab => enigo::Key::Tab,
        rdev::Key::UpArrow => enigo::Key::UpArrow,
        rdev::Key::PrintScreen => enigo::Key::Print,
        rdev::Key::ScrollLock => enigo::Key::ScrollLock,
        rdev::Key::Pause => enigo::Key::Pause,
        rdev::Key::NumLock => enigo::Key::Numlock,
        rdev::Key::BackQuote => todo!(),
        rdev::Key::Num1 => enigo::Key::Unicode('1'),
        rdev::Key::Num2 => enigo::Key::Unicode('2'),
        rdev::Key::Num3 => enigo::Key::Unicode('3'),
        rdev::Key::Num4 => enigo::Key::Unicode('4'),
        rdev::Key::Num5 => enigo::Key::Unicode('5'),
        rdev::Key::Num6 => enigo::Key::Unicode('6'),
        rdev::Key::Num7 => enigo::Key::Unicode('7'),
        rdev::Key::Num8 => enigo::Key::Unicode('8'),
        rdev::Key::Num9 => enigo::Key::Unicode('9'),
        rdev::Key::Num0 => enigo::Key::Unicode('0'),
        rdev::Key::Minus => enigo::Key::Unicode('-'),
        rdev::Key::Equal => enigo::Key::Unicode('='),
        rdev::Key::KeyQ => enigo::Key::Unicode('q'),
        rdev::Key::KeyW => enigo::Key::Unicode('w'),
        rdev::Key::KeyE => enigo::Key::Unicode('e'),
        rdev::Key::KeyR => enigo::Key::Unicode('r'),
        rdev::Key::KeyT => enigo::Key::Unicode('t'),
        rdev::Key::KeyY => enigo::Key::Unicode('y'),
        rdev::Key::KeyU => enigo::Key::Unicode('u'),
        rdev::Key::KeyI => enigo::Key::Unicode('i'),
        rdev::Key::KeyO => enigo::Key::Unicode('o'),
        rdev::Key::KeyP => enigo::Key::Unicode('p'),
        rdev::Key::LeftBracket => enigo::Key::Unicode('('),
        rdev::Key::RightBracket => enigo::Key::Unicode(')'),
        rdev::Key::KeyA => enigo::Key::Unicode('a'),
        rdev::Key::KeyS => enigo::Key::Unicode('s'),
        rdev::Key::KeyD => enigo::Key::Unicode('d'),
        rdev::Key::KeyF => enigo::Key::Unicode('f'),
        rdev::Key::KeyG => enigo::Key::Unicode('g'),
        rdev::Key::KeyH => enigo::Key::Unicode('h'),
        rdev::Key::KeyJ => enigo::Key::Unicode('j'),
        rdev::Key::KeyK => enigo::Key::Unicode('k'),
        rdev::Key::KeyL => enigo::Key::Unicode('l'),
        rdev::Key::SemiColon => enigo::Key::Unicode(';'),
        rdev::Key::Quote => enigo::Key::Unicode('"'), // HUH ?
        rdev::Key::BackSlash => enigo::Key::Unicode('\\'),
        rdev::Key::IntlBackslash => enigo::Key::Unicode('\\'),
        rdev::Key::KeyZ => enigo::Key::Unicode('z'),
        rdev::Key::KeyX => enigo::Key::Unicode('x'),
        rdev::Key::KeyC => enigo::Key::Unicode('c'),
        rdev::Key::KeyV => enigo::Key::Unicode('v'),
        rdev::Key::KeyB => enigo::Key::Unicode('b'),
        rdev::Key::KeyN => enigo::Key::Unicode('n'),
        rdev::Key::KeyM => enigo::Key::Unicode('m'),
        rdev::Key::Comma => enigo::Key::Unicode(','),
        rdev::Key::Dot => enigo::Key::Unicode('.'),
        rdev::Key::Slash => enigo::Key::Unicode('/'),
        rdev::Key::Insert => enigo::Key::Insert,
        rdev::Key::KpReturn => todo!(),
        rdev::Key::KpMinus => todo!(),
        rdev::Key::KpPlus => todo!(),
        rdev::Key::KpMultiply => todo!(),
        rdev::Key::KpDivide => todo!(),
        rdev::Key::Kp0 => todo!(),
        rdev::Key::Kp1 => todo!(),
        rdev::Key::Kp2 => todo!(),
        rdev::Key::Kp3 => todo!(),
        rdev::Key::Kp4 => todo!(),
        rdev::Key::Kp5 => todo!(),
        rdev::Key::Kp6 => todo!(),
        rdev::Key::Kp7 => todo!(),
        rdev::Key::Kp8 => todo!(),
        rdev::Key::Kp9 => todo!(),
        rdev::Key::KpDelete => todo!(),
        rdev::Key::Function => todo!(),
        rdev::Key::Unknown(_) => todo!(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel(100);
    let tx_clone = tx.clone();

    // Spawn a thread to listen for events
    std::thread::spawn(move || {
        listen(move |event| {
            if let Err(_) = tx_clone.blocking_send(event) {
                // Receiver dropped
                return;
            }
        })
        .unwrap();
    });

    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server");

    while let Some(event) = rx.recv().await {
        let cmd = match event.event_type {
            EventType::MouseMove { x, y } => Command::MouseMove {
                x: x as i32,
                y: y as i32,
                coord: Coordinate::Abs,
            },
            EventType::ButtonRelease(button) => Command::MouseRelease {
                button: map_mouse_buttons(button),
            },
            EventType::ButtonPress(button) => Command::MousePress {
                button: map_mouse_buttons(button),
            },
            EventType::KeyPress(key) => Command::KeyPress {
                key: map_keyboard_keys(key),
            },
            EventType::KeyRelease(key) => Command::KeyRelease {
                key: map_keyboard_keys(key),
            },
            _ => continue, // Ignore other events
        };

        let serialized = serde_json::to_string(&cmd)?;
        stream.write_all(serialized.as_bytes()).await?;
        stream.write_all(b"\n").await?;
    }

    Ok(())
}
