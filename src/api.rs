use rocket_contrib::{JSON, Value};

use std::sync::atomic::*;
use std::sync::mpsc;
use std::thread;
use ws;

#[derive(Debug, Serialize)]
pub struct RegisterPlayerResponse {
    id: usize,
}

#[get("/register-player")]
pub fn register_player() -> JSON<RegisterPlayerResponse> {
    static PLAYER_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

    let next_id = PLAYER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    // TODO: Update game state to reflect the registered player.

    JSON(RegisterPlayerResponse {
        id: next_id,
    })
}

#[derive(Debug, Serialize)]
pub enum Broadcast {
    Host(HostBroadcast),
    Player(PlayerBroadcast),
}

#[derive(Debug, Serialize)]
pub struct HostBroadcast;

#[derive(Debug, Serialize)]
pub struct PlayerBroadcast;

/// Spawns the websocket server, returning a sender for broadcasting messages.
///
/// The websocket server is run on a separate thread listening on port 6768. This is a necessary
/// workaround because Rocket doesn't yet directly support websockets. The returned `mpsc::Sender`
/// allows for API messages to be sent from any number of threads to the websocket server, at which
/// point they will be broadcast to any connected clients.
pub fn start_websocket_server() -> mpsc::Sender<Broadcast> {
    // Create a sender/reciever pair so that the Rocket server can send messages to be broadcast
    // to all websocket listeners.
    let (sender, receiver) = mpsc::channel();

    // Spawn a thread to host the websocket server. The websocket server must listen on a different
    // port than the Rocket server, so we use 6768. The thread takes the receiver end of the
    // channel so that it can pull messages out of the queue and broadcast them to any active
    // hosts/clients.
    thread::spawn(|| {
        // let _ = receiver; // TODO: Actually use the receiver.
        ws::listen("localhost:6768", |out| {
            println!("Made a new websocket connection");

            // TODO: Listen for input from the receiver and pump that shit out to the websockets.
            |message| {
                println!("Recieved message from socket: {:?}", message);
                Ok(())
            }
        }).expect("Websocket ");
    });

    sender
}
