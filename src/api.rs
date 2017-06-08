use rocket_contrib::{JSON, Value};

use rocket::State;
use std::sync::atomic::*;
use std::sync::*;
use std::thread;
use ws;

/// Uniquely identifies a connected player.
///
/// When a new player joins, they use the `/api/register-player` endpoint to register themselves.
/// Registration generates a new `PlayerId`, which is stored inside the server and returned to the
/// client. If the client disconnects and wants to rejoin, they can continue using the previous
/// `PlayerId` to avoid losing the player's progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct PlayerId(usize);

/// Generator for `PlayerId`.
///
/// Meant to be managed as application state by Rocket. Only one should ever be created, and Rocket
/// ensures that only one can ever be registered as managed state.
#[derive(Debug)]
pub struct PlayerIdGenerator(AtomicUsize);

impl PlayerIdGenerator {
    /// Creates a new `PlayerIdGenerator`.
    ///
    /// Only one `PlayerIdGenerator` should be created in the lifetime of the application. A single
    /// generator will never create duplicate IDs, but if there are multiple generators will
    /// produce the same IDs.
    pub fn new() -> PlayerIdGenerator {
        PlayerIdGenerator(ATOMIC_USIZE_INIT)
    }

    /// Generate a unique ID for a player.
    pub fn next_id(&self) -> PlayerId {
        PlayerId(self.0.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Serialize)]
pub struct RegisterPlayerResponse {
    id: PlayerId,
}

#[get("/register-player")]
pub fn register_player(player_id_generator: State<PlayerIdGenerator>, broadcast_sender: State<Mutex<mpsc::Sender<Broadcast>>>) -> JSON<RegisterPlayerResponse> {
    let next_id = player_id_generator.next_id();

    // TODO: Update game state to reflect the registered player.

    // Broadcast to all hosts that a new player has joined.
    broadcast_sender
        .lock()
        .expect("Failed to acquire lock on broadcast sender")
        .send(Broadcast::Host(HostBroadcast::PlayerRegistered(next_id)))
        .expect("Failed to send player register broadcast");

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
pub enum HostBroadcast {
    PlayerRegistered(PlayerId),
}

#[derive(Debug, Serialize)]
pub struct PlayerBroadcast;

/// Spawns the websocket server, returning a sender for broadcasting messages.
///
/// The websocket server is run on a separate thread listening on port 6768. This is a necessary
/// workaround because Rocket doesn't yet directly support websockets. The returned `mpsc::Sender`
/// allows for API messages to be sent from any number of threads to the websocket server, at which
/// point they will be broadcast to any connected clients.
pub fn start_websocket_server() -> mpsc::Sender<Broadcast> {
    use ws::*;

    // Create a sender/reciever pair so that the Rocket server can send messages to be broadcast
    // to all websocket listeners.
    let (broadcast_sender, broadcast_receiver) = mpsc::channel();
    let (socket_sender, socket_receiver) = mpsc::channel();

    // Spawn a thread to host the websocket server. The websocket server must listen on a different
    // port than the Rocket server, so we use 6768. As websocket connections are opened, the sender
    // end of the connection is saved so that
    thread::spawn(move || {
        ws::listen("localhost:6768", |ws_sender| {
            println!("Made a new websocket connection: {:?}", ws_sender.token());

            // Send the websocket sender to the broadcast thread.
            socket_sender.send(ws_sender);

            // Create a noop handler for the connection. We don't care about listening for messages
            // or doing any advanced handling (for now, at least), so we don't need the handler to
            // to do anything.
            |_| { Ok(()) }
        });
    });

    // Spawn a thread to read broadcasts and multiplex them to the websockets.
    thread::spawn(move || {
        let mut sockets = Vec::new();

        for broadcast in broadcast_receiver {
            // Grab any pending sockets that have been sent.
            for socket in socket_receiver.try_iter() {
                sockets.push(socket);
            }

            // Serialize the broadcast as JSON.
            let payload = ::serde_json::to_string(&broadcast).expect("Failed to serialize payload");

            for socket in &sockets {
                // TODO: If `send` returns an `Err`, then it means the socket closed (right?). We
                // should remove closed sockets from `sockets`.
                socket.send(payload.clone());
            }

            // TODO: Split messages between host broadcasts and player broadcasts.
            // Broadcast the message out to all sockets.
            // match broadcast {
            //     Broadcast::Host(host_broadcast) => {
            //         for host_socket in host_sockets {
            //             host_socket.send(host_broadcast);
            //         }
            //     }
            //
            //     Broadcast::Player(player_broadcast) => {
            //         for player_socket in player_sockets {
            //             player_socket.send(player_broadcast);
            //         }
            //     }
            // }
        }
    });

    broadcast_sender
}
