#![allow(unused)]
//! TODO: implement a simple chat server
//!
//! The chat server will allow users to connect to it through TCP/IP, set their username,
//! and then send either direct messages (DMs) or broadcasts to other connected users.
//! The server should properly handle graceful shutdown and client disconnects, and avoid
//! interleaving unrelated messages.
//! It should also support concurrency and allow the connection of multiple clients at once.
//!
//! You do not need to implement message encoding and network communication details, as those have
//! already been implemented for you (see `reader.rs` and `writer.rs`).
//!
//! **Do not use `async/await` or any external crates that deal with networking for this assignment.
//! The existing dependencies of this crate (`anyhow`, `serde`, `serde_json`) should be enough.**
//!
//! Try to distribute your code across multiple files (modules), based on the responsibility of
//! the code (code that deals with the same stuff should generally be in the same module).
//!
//! Hint: take a look at the [`TcpStream::shutdown`] function, which can be used to terminate
//! a TCP/IP connection. It might be useful here :)
//!
//! Note: this assignment will probably get extended in the upcoming weeks, so it would be nice if
//! you implement at least some part of it, so that you can continue improving it later.

use core::error;
use std::{borrow::Borrow, clone, fs::read, io::{Read, Write}, net::{IpAddr, Ipv4Addr, TcpListener, TcpStream}, os::unix::{net::SocketAddr, thread}, sync::{atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering}, Arc, Mutex}, thread::JoinHandle, time::Duration};

use anyhow::Ok;
use messages::{ClientToServerMsg, ServerToClientMsg};
use reader::MessageReader;
use socket_wrapper::SocketWrapper;
use writer::MessageWriter;

/// The following modules were prepared for you. You should not need to modify them.
///
/// Take a look at this file to see how should the individual messages be handled
mod messages;
/// Message reading
mod reader;
/// Message writing
mod writer;
/// Socket wrapper
mod socket_wrapper;

#[derive(Copy, Clone)]
struct ServerOpts {
    /// Maximum number of clients that can be connected to the server at once.
    max_clients: usize,
}

type VecAM<T> = Arc<Mutex<Vec<T>>>;
struct RunningServer {
    pub server: TcpListener,
    pub handle: Option<std::thread::JoinHandle<Result<(), anyhow::Error>>>,
    pub connected_clients: VecAM<ConnectedClientAM>,
    pub port: u16,
    pub clients_count: Arc<AtomicUsize>,
    pub stop_command: Arc<AtomicBool>,
}

impl RunningServer {
    fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for RunningServer {
    fn drop(&mut self) {
        self.stop_command.store(true, Ordering::Relaxed);
        println!("Sent stopping command");
        if let Some(handle) = self.handle.take() {
            if let Err(err) = handle.join() {
                eprintln!("Error joining thread: {:?}", err);
            }
        }
    }
}


struct ConnectedClientAM(Arc<Mutex<ConnectedClient>>); 
impl ConnectedClientAM {
    fn get_name(&self) -> String {
        self.0.lock().unwrap().name.clone()
    }

    fn get_id(&self) -> usize {
        self.0.lock().unwrap().id
    }

    fn shutdown_connection(&mut self) {
        self.0.lock().unwrap().connection.shutdown(std::net::Shutdown::Both).unwrap();
    }

    fn write(&mut self, data: ServerToClientMsg) -> anyhow::Result<()>{
        self.0.lock().unwrap().writer.write(data)
    }

    fn disconnect(&mut self,
        connection: Arc<TcpStream>,
        clients_count: Arc<AtomicUsize>,
        connected_clients: VecAM<ConnectedClientAM>
    ) {
        let client_id = self.get_id();
        self.0.lock().unwrap().disconnect();
        connected_clients.lock().unwrap().retain(|x| x.get_id() != client_id);
        clients_count.fetch_sub(1, Ordering::SeqCst);
    }
}

impl From<ConnectedClient> for ConnectedClientAM {
    fn from(value: ConnectedClient) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}

impl Clone for ConnectedClientAM {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct ConnectedClient {
    // pub handle: JoinHandle<()>,
    pub connection: Arc<TcpStream>,
    pub writer: MessageWriter::<ServerToClientMsg, SocketWrapper>,

    pub id: usize,
    pub name: String,
}

impl ConnectedClient {
    fn disconnect(&mut self) {
        self.connection.shutdown(std::net::Shutdown::Both);
    }

    
}

/// TODO: implement the following function called `run_server`
/// It should start a chat server on a TCP/IP port assigned to it by the operating system and
/// return a structure called `RunningServer`. This struct should have a method called `port`,
/// which returns the port on which the server is running.
///
/// The server should implement the messages described in `messages.rs`, see the message comments
/// for more details.
///
/// # Client connection
/// When a client connects to the server, it should send a `Join` message.
/// If it sends anything else, the server should respond with an error "Unexpected message received"
/// and disconnect the client immediately.
/// If the user sends a Join message (with a unique username), the server should respond with
/// the `Welcome` message.
/// Then it should start receiving requests from the client.
/// If the client ever sends the `Join` message again, the server should respond with an error
/// "Unexpected message received" and disconnect the client immediately.
///
/// # Maximum number of clients
/// When a client connects and there are already `opts.max_clients` other clients connected, the
/// server should respond with an error "Server is full" and disconnect the client immediately.
/// Note that if the server is full, the client should be disconnected even before it sends the
/// `Join` message.
///
/// # Graceful shutdown
/// When `RunningServer` is dropped, it should:
/// 1) Stop receiving new TCP/IP connections
/// 2) Correctly disconnect all connected users
/// 3) Wait until all threads that it has created has completed executing
///
/// Graceful shutdown with threads and blocking I/O is challenging (if you don't consider
/// `exit()` or `abort()` to be a "graceful" shutdown :) ), because it can be difficult to
/// communicate with blocked threads.
/// Think about how you can get around this - can you find some way to "wake" the threads up?
///
/// See tests for more details.

fn handle_server(
    stop_command_clone: Arc<AtomicBool>,
    clients_count_clone: Arc<AtomicUsize>,
    server_clone: TcpListener,
    connected_clients_clone: VecAM<ConnectedClientAM>,
    client_id_counter_clone: Arc<AtomicUsize>,
    max_clients: usize,
) -> anyhow::Result<()> {
    let mut handles = Vec::new();
    while !stop_command_clone.load(Ordering::SeqCst) {
        match server_clone.accept() {
            Result::Ok((accepted_connection, _)) => {
                let accepted_connection = Arc::new(accepted_connection);
                let stop_command_clone2 = stop_command_clone.clone();
                let clients_count_clone2 = clients_count_clone.clone();
                let connected_clients_clone2 = connected_clients_clone.clone();
                let client_id_counter_clone2 = client_id_counter_clone.clone();

                handles.push(std::thread::spawn(move || handle_client_thread(client_id_counter_clone2, connected_clients_clone2, accepted_connection, clients_count_clone2, stop_command_clone2, max_clients)));  
            }
            Result::Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Result::Err(e) => {
                eprintln!("Error accepting connection: {:?}", e);
            }
        }
    }
    println!("Received stopping command");
    for connection in connected_clients_clone.lock().unwrap().iter_mut() {
        connection.shutdown_connection();
    }

    for handle2 in handles {
        handle2.join().unwrap();
    }
    Ok(()) 
}

fn handle_client_thread(
    client_id_counter: Arc<AtomicUsize>,
    connected_clients: VecAM<ConnectedClientAM>,
    accepted_connection: Arc<TcpStream>,
    clients_count: Arc<AtomicUsize>,
    stop_command: Arc<AtomicBool>, 
    max_clients: usize,
) {
    let mut message_reader = MessageReader::<ClientToServerMsg, SocketWrapper>::new(
        SocketWrapper(accepted_connection.clone())
    );
    let mut message_writer = MessageWriter::<ServerToClientMsg, SocketWrapper>::new(
        SocketWrapper(accepted_connection.clone())
    );

    if clients_count.load(Ordering::SeqCst) >= max_clients {
        message_writer.write(ServerToClientMsg::Error("Server is full".to_string())).unwrap();
        accepted_connection.shutdown(std::net::Shutdown::Both).unwrap();
        return;
    }
    clients_count.fetch_add(1, Ordering::SeqCst);

    let thread_id = client_id_counter.fetch_add(1, Ordering::SeqCst);

    let username = match authenticate_user(&mut message_reader, &mut message_writer, &mut connected_clients.clone()) {
        Some(name) => name,
        None => {
            disconnect_client(accepted_connection, clients_count, connected_clients, thread_id);
            return;
        },
    };

    let mut client_data: ConnectedClientAM = ConnectedClient {
        id: thread_id,
        name: username.clone(),
        writer: message_writer,
        connection: accepted_connection.clone()
    }.into();
    connected_clients.lock().unwrap().push(client_data.clone());

    loop {
        if stop_command.load(Ordering::SeqCst) {
            break;
        }
        match message_reader.next() {
            Some(message) => {
                match message {
                    Result::Ok(data) => {
                        match data {
                            ClientToServerMsg::Join { name } => {
                                client_data.write(ServerToClientMsg::Error("Unexpected message received".to_string()));
                                client_data.disconnect(accepted_connection, clients_count, connected_clients);
                                return;
                            },
                            ClientToServerMsg::Ping => {
                                client_data.write(ServerToClientMsg::Pong);
                            },
                            ClientToServerMsg::ListUsers => {
                                let users = connected_clients.lock().unwrap()
                                    .iter()
                                    .map(|user| user.get_name())
                                    .collect::<Vec<String>>();
                                client_data.write(ServerToClientMsg::UserList {
                                    users
                                });
                            },
                            ClientToServerMsg::SendDM { to, message } => {
                                if to == username {
                                    client_data.write(ServerToClientMsg::Error("Cannot send a DM to yourself".to_string()));
                                }

                                let mut clients_lock = connected_clients.lock().unwrap();
                                match clients_lock.iter_mut().find(|x| x.get_name() == to) {
                                    Some(user) => {
                                        user.write(ServerToClientMsg::Message { from: username.clone(), message });
                                    },
                                    None => {
                                        client_data.write(ServerToClientMsg::Error(format!("User {} does not exist", to)));
                                    },
                                }
                            },
                            ClientToServerMsg::Broadcast { message } => {
                                let mut a = connected_clients.lock().unwrap();
                                for user in a.iter_mut() {
                                    if user.get_name() != username {
                                        user.write(ServerToClientMsg::Message { from: username.clone(), message: message.clone() });
                                    }
                                }
                            },
                        }
                    },
                    Result::Err(error_data) => {
                        if let Some(e) = error_data.downcast_ref::<std::io::Error>() {
                            if e.kind() == std::io::ErrorKind::WouldBlock && stop_command.load(Ordering::SeqCst) {
                                return;
                            }
                        }
                    },
                }
            },
            None => {
                client_data.disconnect(accepted_connection, clients_count, connected_clients);
                println!("Disconnected");
                return;
            },
        }
    }
}

fn disconnect_client(
    connection: Arc<TcpStream>,
    clients_count: Arc<AtomicUsize>,
    connected_clients: VecAM<ConnectedClientAM>,
    client_id: usize
) {
    connection.shutdown(std::net::Shutdown::Both);
    clients_count.fetch_sub(1, Ordering::SeqCst);
    connected_clients.lock().unwrap().retain(|x| x.get_id() != client_id);
}

fn authenticate_user(
    reader: &mut MessageReader::<ClientToServerMsg, SocketWrapper>,
    writer: &mut MessageWriter::<ServerToClientMsg, SocketWrapper>,
    connected_clients: &mut VecAM<ConnectedClientAM>,
) -> Option<String> { 
    match reader.next() {
        Some(message) => {
            match message {
                Result::Ok(data) => {
                    match data {
                        ClientToServerMsg::Join { name } => {
                            let a = connected_clients.lock().unwrap();
                            let b = a.iter().find(|x| x.get_name() == name);
                            match b {
                                Some(_) => {
                                    writer.write(ServerToClientMsg::Error("Username already taken".to_string()));
                                    None
                                }
                                None => {
                                    writer.write(ServerToClientMsg::Welcome);
                                    Some(name)
                                },
                            }
                        },
                        _ => {
                            writer.write(ServerToClientMsg::Error("Unexpected message received".to_string()));
                            None
                        }
                    }
                },
                Result::Err(err_data) => {
                    None
                },
            }
        },
        None => {
            None
        },
    }
}

fn run_server(opts: ServerOpts) -> anyhow::Result<RunningServer> { 
    let server = TcpListener::bind("127.0.0.1:0")?;
    server.set_nonblocking(true)?;

    let client_id_counter = Arc::new(AtomicUsize::new(0));

    let clients_count = Arc::new(AtomicUsize::new(0));
    let port = server.local_addr().unwrap().port();
    let stop_command = Arc::new(AtomicBool::new(false));
    let connected_clients: VecAM<ConnectedClientAM> = Arc::new(Mutex::new(Vec::new()));
    

    let stop_command_clone = stop_command.clone();
    let clients_count_clone = clients_count.clone();
    let server_clone = server.try_clone().unwrap();
    let connected_clients_clone = connected_clients.clone();
    let client_id_counter_clone = client_id_counter.clone();
    
    let mut rs = RunningServer {
        server,
        handle: None,
        port,
        stop_command,
        clients_count,
        connected_clients,
    };
    
    let handle = std::thread::spawn(move || handle_server(stop_command_clone, clients_count_clone, server_clone, connected_clients_clone, client_id_counter_clone, opts.max_clients));


    rs.handle = Some(handle);

    Ok(rs)
}


#[cfg(test)]
mod tests {
    use crate::messages::{ClientToServerMsg, ServerToClientMsg};
    use crate::reader::MessageReader;
    use crate::socket_wrapper::SocketWrapper;
    use crate::writer::MessageWriter;
    use crate::{run_server, RunningServer, ServerOpts};
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpStream};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread::spawn;
    use std::time::Duration;

    #[test]
    fn empty_server_shuts_down() {
        run_test(opts(2), |_| Ok(()));
    }

    #[test]
    fn max_clients() {
        run_test(opts(2), |server| {
            let _client = server.client();
            let _client2 = server.client();

            let mut client3 = server.client();
            client3.expect_error("Server is full");
            client3.check_closed();

            Ok(())
        });
    }

    #[test]
    fn max_clients_after_client_leaves() {
        run_test(opts(2), |server| {
            let _client = server.client();
            let client2 = server.client();
            client2.close();

            sleep(1000);

            let mut client3 = server.client();
            client3.join("Foo");

            Ok(())
        });
    }

    #[test]
    fn max_clients_herd() {
        let max_clients = 5;
        run_test(opts(max_clients), |server| {
            let thread_count = 50;

            let server = Arc::new(server);
            let barrier = Arc::new(Barrier::new(thread_count));

            let errors = Arc::new(AtomicUsize::new(0));
            let successes = Arc::new(AtomicUsize::new(0));

            let joined_clients = Arc::new(Mutex::new(vec![]));
            std::thread::scope(|s| {
                for thread_id in 0..thread_count {
                    let barrier = barrier.clone();
                    let server = server.clone();
                    let errors = errors.clone();
                    let successes = successes.clone();
                    let joined_clients = joined_clients.clone();
                    s.spawn(move || {
                        barrier.wait();
                        let mut client = server.client();
                        client.send(ClientToServerMsg::Join {
                            name: format!("Thread {thread_id}"),
                        });
                        match client.recv() {
                            ServerToClientMsg::Error(_) => {
                                errors.fetch_add(1, Ordering::SeqCst);
                            }
                            ServerToClientMsg::Welcome => {
                                successes.fetch_add(1, Ordering::SeqCst);
                                // Make sure that the client doesn't disconnect
                                joined_clients.lock().unwrap().push(client);
                            }
                            msg => {
                                panic!("Unexpected message {msg:?}");
                            }
                        }
                    });
                }
            });
            assert_eq!(errors.load(Ordering::SeqCst), thread_count - max_clients);
            assert_eq!(successes.load(Ordering::SeqCst), max_clients);

            drop(joined_clients);

            Ok(())
        });
    }

    #[test]
    fn list_users_before_join() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.send(ClientToServerMsg::ListUsers);
            client.expect_error("Unexpected message received");

            Ok(())
        });
    }

    #[test]
    fn duplicated_join() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.join("Foo");
            client.send(ClientToServerMsg::Join {
                name: "Bar".to_string(),
            });
            client.expect_error("Unexpected message received");

            Ok(())
        });
    }

    #[test]
    fn error_then_disconnect() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.join("Foo");
            client.send(ClientToServerMsg::Join {
                name: "Bar".to_string(),
            });
            client.close();

            let mut client2 = server.client();
            client2.join("Bar");

            Ok(())
        });
    }

    #[test]
    fn duplicated_username() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.join("Foo");

            let mut client2 = server.client();
            client2.send(ClientToServerMsg::Join {
                name: "Foo".to_string(),
            });
            client2.expect_error("Username already taken");

            Ok(())
        });
    }

    #[test]
    fn ping() {
        run_test(opts(2), |server| {
            let mut luca = server.client();
            luca.join("Luca");
            luca.ping();

            Ok(())
        });
    }

    #[test]
    fn ping_before_join() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.send(ClientToServerMsg::Ping);
            client.expect_error("Unexpected message received");

            Ok(())
        });
    }

    #[test]
    fn list_users_reconnect() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.join("Foo");
            client.close();

            let mut client = server.client();
            client.join("Foo");
            assert_eq!(client.list_users(), vec!["Foo".to_string()]);

            Ok(())
        });
    }

    #[test]
    fn list_users_self() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.join("Martin");
            assert_eq!(client.list_users(), vec!["Martin".to_string()]);

            Ok(())
        });
    }

    #[test]
    fn list_users_ignore_not_joined_users() {
        run_test(opts(2), |server| {
            let _client = server.client();
            let mut client2 = server.client();
            client2.join("Joe");
            assert_eq!(client2.list_users(), vec!["Joe".to_string()]);

            Ok(())
        });
    }

    #[test]
    fn list_users_after_error() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.join("Terrence");

            let mut client2 = server.client();
            client2.join("Joe");

            client.send(ClientToServerMsg::Join {
                name: "Barbara".to_string(),
            });

            sleep(1000);

            assert_eq!(client2.list_users(), vec!["Joe".to_string()]);

            Ok(())
        });
    }

    #[test]
    fn list_users() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.join("Terrence");

            let mut client2 = server.client();
            client2.join("Joe");
            assert_eq!(
                client2.list_users(),
                vec!["Joe".to_string(), "Terrence".to_string()]
            );
            client2.close();

            assert_eq!(client.list_users(), vec!["Terrence".to_string()]);

            Ok(())
        });
    }

    #[test]
    fn dm_nonexistent_user() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.join("Mark");
            client.dm("Fiona", "Hi");
            client.expect_error("User Fiona does not exist");

            Ok(())
        });
    }

    #[test]
    fn dm_self() {
        run_test(opts(2), |server| {
            let mut client = server.client();
            client.join("Xal'atath");
            client.dm("Xal'atath", "I'm so lonely :(");
            client.expect_error("Cannot send a DM to yourself");

            Ok(())
        });
    }

    #[test] // error
    fn dm_other() {
        run_test(opts(2), |server| {
            let mut terrence = server.client();
            terrence.join("Terrence");

            let mut joe = server.client();
            joe.join("Joe");

            terrence.dm("Joe", "How you doin'");
            joe.expect_message("Terrence", "How you doin'");

            Ok(())
        });
    }

    #[test]
    fn dm_spam() {
        run_test(opts(2), |server| {
            let mut diana = server.client();
            diana.join("Diana");

            let mut francesca = server.client();
            francesca.join("Francesca");

            let barrier = Arc::new(Barrier::new(2));
            let barrier2 = barrier.clone();

            let count = 100000;

            // Let's say that someone is spamming you...
            let t1 = spawn(move || {
                barrier.wait();

                for _ in 0..count {
                    diana.dm("Francesca", "Can I borrow your brush? Pleeeeeease :(((");
                }
            });

            // ...so you get angry, and start spamming them back.
            // But you make a critical *error*, because you're sending the message
            // to the wrong account.
            // Can your chat server handle that?
            let t2 = spawn(move || {
                // Sync the threads a little bit
                barrier2.wait();

                for _ in 0..count {
                    francesca.dm("Daina", "NO! Get your own!");
                    match francesca.recv() {
                        ServerToClientMsg::Message { from, message } => {
                            assert_eq!(from, "Diana");
                            assert_eq!(message, "Can I borrow your brush? Pleeeeeease :(((");
                        }
                        ServerToClientMsg::Error(error) => {
                            assert_eq!(error, "User Daina does not exist");
                        }
                        msg => panic!("Unexpected message {msg:?}"),
                    }
                }
                // Francesca should receive count * 2 messages, `count` from Diana and `count`
                // error messages
                for _ in 0..count {
                    match francesca.recv() {
                        ServerToClientMsg::Message { from, message } => {
                            assert_eq!(from, "Diana");
                            assert_eq!(message, "Can I borrow your brush? Pleeeeeease :(((");
                        }
                        ServerToClientMsg::Error(error) => {
                            assert_eq!(error, "User Daina does not exist");
                        }
                        msg => panic!("Unexpected message {msg:?}"),
                    }
                }
            });
            t1.join().unwrap();
            t2.join().unwrap();

            Ok(())
        });
    }

    #[test] // todo
    fn dm_spam_2() {
        // Meanwhile, in a parallel universe...
        run_test(opts(2), |server| {
            let mut diana = server.client();
            diana.join("Diana");

            let mut francesca = server.client();
            francesca.join("Francesca");

            let barrier = Arc::new(Barrier::new(2));
            let barrier2 = barrier.clone();

            let count = 100000;

            // Let's say that someone is spamming you...
            let t1 = spawn(move || {
                barrier.wait();

                for _ in 0..count {
                    diana.dm("Francesca", "Can I borrow your brush? Pleeeeeease :(((");
                }
            });

            // ...so you get angry, and start spamming them back.
            // But you make a critical *error*, because you push the wrong button and start
            // sending pings to the server instead.
            // Can your chat server handle that?
            let t2 = spawn(move || {
                // Sync the threads a little bit
                barrier2.wait();

                for _ in 0..count {
                    francesca.send(ClientToServerMsg::Ping);
                    match francesca.recv() {
                        ServerToClientMsg::Message { from, message } => {
                            assert_eq!(from, "Diana");
                            assert_eq!(message, "Can I borrow your brush? Pleeeeeease :(((");
                        }
                        ServerToClientMsg::Pong => {}
                        msg => panic!("Unexpected message {msg:?}"),
                    }
                }
                // Francesca should receive count * 2 messages, `count` from Diana and `count`
                // pong messages
                for _ in 0..count {
                    match francesca.recv() {
                        ServerToClientMsg::Message { from, message } => {
                            assert_eq!(from, "Diana");
                            assert_eq!(message, "Can I borrow your brush? Pleeeeeease :(((");
                        }
                        ServerToClientMsg::Pong => {}
                        msg => panic!("Unexpected message {msg:?}"),
                    }
                }
            });
            t2.join().unwrap();
            t1.join().unwrap();

            Ok(())
        });
    }

    #[test]
    fn broadcast_empty() {
        run_test(opts(2), |server| {
            let mut ji = server.client();
            ji.join("Ji");
            ji.send(ClientToServerMsg::Broadcast {
                message: "Haaaaaai!".to_string(),
            });
            ji.ping();

            Ok(())
        });
    }

    #[test]
    fn broadcast() {
        run_test(opts(10), |server| {
            let mut niko = server.client();
            niko.join("Niko");

            let users: Vec<Client> = (0..5)
                .map(|i| {
                    let mut client = server.client();
                    client.join(&format!("NPC {i}"));
                    client
                })
                .collect();

            niko.send(ClientToServerMsg::Broadcast {
                message: "Borrow this!".to_string(),
            });
            niko.ping();

            for mut user in users {
                user.expect_message("Niko", "Borrow this!");
            }

            Ok(())
        });
    }

    // TODO(bonus): uncomment the following test and make it pass
    // The server should correctly close client socket when it shuts down,
    // to avoid a situation where the clients would be stuck waiting for a message
    // for some indeterminate amount of time.
    #[test]
    fn drop_clients_on_shutdown() {
        let server = run_server(opts(2)).expect("creating server failed");

        let mut client = server.client();
        client.join("Bar");
        let mut client2 = server.client();
        client2.join("Foo");

        drop(server);

        assert!(client.reader.read().is_none());
        assert!(client2.reader.read().is_none());
    }
    
    fn run_test<F: FnOnce(RunningServer) -> anyhow::Result<()>>(opts: ServerOpts, func: F) {
        let server = run_server(opts).expect("creating server failed");
        let port = server.port;
        func(server).expect("test failed");

        TcpStream::connect(("127.0.0.1", port)).expect_err("server is still alive");
    }

    struct Client {
        writer: MessageWriter<ClientToServerMsg, SocketWrapper>,
        reader: MessageReader<ServerToClientMsg, SocketWrapper>,
    }

    impl Client {
        #[track_caller]
        fn join(&mut self, name: &str) {
            self.send(ClientToServerMsg::Join {
                name: name.to_string(),
            });
            let msg = self.recv();
            assert!(matches!(msg, ServerToClientMsg::Welcome));
        }

        #[track_caller]
        fn ping(&mut self) {
            self.send(ClientToServerMsg::Ping);
            let msg = self.recv();
            assert!(matches!(msg, ServerToClientMsg::Pong));
        }

        #[track_caller]
        fn list_users(&mut self) -> Vec<String> {
            self.send(ClientToServerMsg::ListUsers);
            let msg = self.recv();
            match msg {
                ServerToClientMsg::UserList { mut users } => {
                    users.sort();
                    users
                }
                msg => {
                    panic!("Unexpected response {msg:?}");
                }
            }
        }

        #[track_caller]
        fn dm(&mut self, to: &str, message: &str) {
            self.send(ClientToServerMsg::SendDM {
                to: to.to_string(),
                message: message.to_string(),
            });
        }

        #[track_caller]
        fn expect_message(&mut self, expected_from: &str, expected_message: &str) {
            let msg = self.recv();
            match msg {
                ServerToClientMsg::Message { from, message } => {
                    assert_eq!(from, expected_from);
                    assert_eq!(message, expected_message);
                }
                msg => panic!("Unexpected message {msg:?}"),
            }
        }

        #[track_caller]
        fn send(&mut self, msg: ClientToServerMsg) {
            self.writer.write(msg).expect("cannot send message");
        }

        #[track_caller]
        fn expect_error(&mut self, expected_error: &str) {
            let msg = self.recv();
            match msg {
                ServerToClientMsg::Error(error) => {
                    assert_eq!(error, expected_error);
                }
                msg => {
                    panic!("Unexpected response {msg:?}");
                }
            }
        }

        fn recv(&mut self) -> ServerToClientMsg {
            self.reader
                .read()
                .expect("connection was closed")
                .expect("did not receive welcome message")
        }

        #[track_caller]
        fn close(self) {
            self.writer.into_inner().0.shutdown(Shutdown::Both).unwrap();
        }

        #[track_caller]
        fn check_closed(mut self) {
            assert!(matches!(self.reader.read(), None | Some(Err(_))));
        }
    }

    impl RunningServer {
        fn client(&self) -> Client {
            let client =
                TcpStream::connect(("127.0.0.1", self.port())).expect("cannot connect to server");
            let client = Arc::new(client);

            let writer = MessageWriter::<ClientToServerMsg, SocketWrapper>::new(SocketWrapper(
                client.clone(),
            ));
            let reader = MessageReader::<ServerToClientMsg, SocketWrapper>::new(SocketWrapper(
                client.clone(),
            ));
            Client { reader, writer }
        }
    }

    fn sleep(duration_ms: u64) {
        std::thread::sleep(Duration::from_millis(duration_ms));
    }

    fn opts(max_clients: usize) -> ServerOpts {
        ServerOpts { max_clients }
    }
}
