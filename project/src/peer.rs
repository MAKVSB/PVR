// Copyright 2018 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
mod network_types;
use enigo::Coordinate;
use network_types::Command;

use rdev::{listen, grab, EventType};
use futures::stream::StreamExt;
use libp2p::{gossipsub, identity::Keypair, mdns, noise, swarm::{NetworkBehaviour, SwarmEvent}, tcp, yamux, PeerId};
use std::error::Error;
use std::time::Duration;
use tokio::{io::{self, AsyncBufReadExt}, sync::mpsc, task};

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
        rdev::Key::ShiftLeft => enigo::Key::LShift,
        rdev::Key::ShiftRight => enigo::Key::Shift,
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


#[cfg(target_os = "linux")]
fn is_host() -> bool {
    return true;
}

#[cfg(target_os = "windows")]
fn is_host() -> bool {
    return false;
}

// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel(100);
    let tx_clone = tx.clone();

    // Spawn a task to read mouse movements and send them through the channel
    task::spawn(async move {
        grab(move |event| {
            if let Err(_) = tx_clone.try_send(event.clone()) {
                // Receiver dropped
                return Some(event);
            };
            Some(event)
        })
        .unwrap();
    });

    let local_key = Keypair::generate_ed25519();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            // Set a custom gossipsub configuration
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

            // build a gossipsub network behaviour
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(MyBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    // Create a Gossipsub topic
    let topic = gossipsub::IdentTopic::new("test-net");
    // subscribes to our topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Listen on all interfaces and whatever port the OS assigns
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    // Kick it off
    loop {
        tokio::select! {
            Ok(Some(line)) = stdin.next_line() => {
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), line.as_bytes()) {
                    println!("Publish error: {e:?}");
                }
            }
            Some(event) = rx.recv() => {
                if is_host() {
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
                        EventType::Wheel { delta_x, delta_y } => Command::MouseWheel {
                            x: delta_x as i32,
                            y: delta_y as i32
                        },
                    };
            
                    let serialized = serde_json::to_string(&cmd)?;


                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), serialized.as_bytes()) {
                        println!("Publish error: {e:?}");
                    };
                }
            },
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(beh) => {
                    match beh {
                        MyBehaviourEvent::Gossipsub(gossip_event) => {
                            match gossip_event {
                                gossipsub::Event::Message { propagation_source, message_id, message } => println!("Got message: '{}' with id: {message_id} from peer: {propagation_source}", String::from_utf8_lossy(&message.data)),
                                gossipsub::Event::Subscribed { peer_id, topic } => println!("Subscribed {} {}", peer_id, topic),
                                gossipsub::Event::Unsubscribed { peer_id, topic } => println!("Unsubscribed {} {}", peer_id, topic),
                                gossipsub::Event::GossipsubNotSupported { peer_id } => println!("GossipsubNotSupported {}", peer_id),
                            }
                        },
                        MyBehaviourEvent::Mdns(mdns_event) => {
                            match mdns_event {
                                mdns::Event::Discovered(vec) => {
                                    for (peer_id, _multiaddr) in vec {
                                        println!("mDNS discovered a new peer: {peer_id}");
                                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                    }
                                },
                                mdns::Event::Expired(vec) => {
                                    for (peer_id, _multiaddr) in vec {
                                        println!("mDNS discover peer has expired: {peer_id}");
                                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                    }
                                },
                            }
                        },
                    }
                },
                SwarmEvent::NewListenAddr { listener_id, address } => {
                    println!("Local node is listening on {address} ({listener_id})");
                },












                SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in } => println!("ConnectionEstabilished {} {} {:?} {} {:?} {:?}", peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in),
                SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => println!("ConnectionClosed {} {} {:?} {} {:?}", peer_id, connection_id, endpoint, num_established, cause),
                SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => println!("IncomingConnection {} {} {}", connection_id, local_addr, send_back_addr),
                SwarmEvent::IncomingConnectionError { connection_id, local_addr, send_back_addr, error } => println!("IncomingConnectionError {} {} {} {} ", connection_id, local_addr, send_back_addr, error ),
                SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error } => println!("OutgoingConnectionError {} {:?} {}", connection_id, peer_id, error),
                SwarmEvent::ExpiredListenAddr { listener_id, address } => println!("ExpiredListenAddr {} {}", listener_id, address),
                SwarmEvent::ListenerClosed { listener_id, addresses, reason } => println!("ListenerClosed {} {:?} {:?}", listener_id, addresses, reason),
                SwarmEvent::ListenerError { listener_id, error } => println!("ListenerError {} {}", listener_id, error),
                SwarmEvent::Dialing { peer_id, connection_id } => println!("Dialing {:?} {}", peer_id, connection_id),
                SwarmEvent::NewExternalAddrCandidate { address } => println!("NewExternalAddrCandidate {}", address),
                SwarmEvent::ExternalAddrConfirmed { address } => println!("ExternalAddrConfirmed {}", address),
                SwarmEvent::ExternalAddrExpired { address } => println!("ExternalAddrExpired {}", address),
                SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => println!("NewExternalAddrOfPeer {} {}", peer_id, address),
                _ => {
                    println!("unidentified event")
                }
            }
        }
    }
}
