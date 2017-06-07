extern crate chan;
extern crate multiqueue;
#[macro_use]
extern crate rouille;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use rouille::{Request, Response, websocket};
use std::collections::HashMap;
use std::thread;
use std::sync::*;
use std::sync::atomic::*;

/// Global client counter, used to generate IDs for new client connections.
///
/// The client counter is incremented for each new client connection, ensuring that each connection
/// is given a unique ID.
// TODO: Create a stronger type for client IDs so they can't get mixed up with regular numbers.
static CLIENT_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

fn main() {
    // Create a channel that can be used to send API call messages from the handler threads to the
    // game thread. We put the sender in an `Arc<Mutex>` so that request handler threads can clone
    // the sender and use it to send the API to the game thread.
    let (api_send, api_recv) = chan::async::<ApiMessage>();

    // Create 2 busses: One for broadcasting updates to the clients, another for broadcasting
    // updates to hosts.
    let (client_send, client_recv) = multiqueue::broadcast_queue(128);
    let client_recv = Mutex::new(client_recv);

    let (host_send, host_recv) = multiqueue::broadcast_queue(32);
    let host_recv = Mutex::new(host_recv);

    // Spawn the game thread, giving it the receiever to use to receive inputs.
    thread::spawn(move || {
        let game_state = GameState {
            scores: HashMap::new(),
        };
        for message in api_recv {
            println!("Received a message from an API endpoint: {:?}", message);

            match message {
                ApiMessage::PlayerRegistered(player_id) => {
                    // TODO: Keep track of the players in some way.
                    println!("Registered player with id: {:?}", player_id);
                    client_send.try_send(ClientUpdate::PlayerRegistered(player_id))
                        .expect("The host broadcast queue was full");
                    host_send.try_send(HostUpdate::PlayerRegistered(player_id))
                        .expect("The host broadcast queue was full");
                },
            }
        }
    });

    println!("Now listening on localhost:6767");
    rouille::start_server("localhost:6767", move |request| {
        router!(request,
            (GET) (/api/register-player) => {
                let player_id = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed);
                api_send.send(ApiMessage::PlayerRegistered(player_id));

                // TODO: Send the player's ID back in the response payload.
                Response::text(format!("{{\"id\": \"{}\"}}", player_id))
            },

            (GET) (/api/client-stream) => {
                let client_broadcast = client_recv
                    .lock()
                    .expect("Unable to lock client receiver")
                    .clone();

                start_broadcast_socket(request, client_broadcast)
                    .expect("Unable to start client socket")
            },

            (GET) (/api/host-stream) => {
                let host_broadcast = host_recv
                    .lock()
                    .expect("Unable to lock host receiver")
                    .clone();

                start_broadcast_socket(request, host_broadcast)
                    .expect("Unable to start host socket")
            },

            // TODO: This redirects the broswer to `/client.html` and `/host.html`, but we don't
            // want the browser to actually show those addresses. Instead, we should serve the file
            // directly without redirecting, that way the URL bar of the browser doesn't show the
            // the change.
            (GET) (/) => { rouille::Response::redirect_303("/client.html") },
            (GET) (/host) => { rouille::Response::redirect_303("/host.html") },
            _ => { rouille::match_assets(&request, "./www/") }
        )
    });
}

/// Creates a websocket from `request` and pumps messages from `broadcast` through the socket.
///
/// Helper method for creating
fn start_broadcast_socket<T>(
    request: &Request,
    broadcast: multiqueue::BroadcastReceiver<T>,
) -> Result<Response, String>
where
    T: 'static + serde::Serialize + Clone + Send,
{
    let (response, receive_websocket) = websocket::start::<String>(&request, None)
        .map_err("Unable to start websocket".into())?;

    thread::spawn(move || {
        // Actually grab the websocket.
        let mut websocket = receive_websocket.recv().expect("Unable to receive from socket");

        // Pump events into the websocket as they become available.
        for event in broadcast {
            // Stop receiving events if the client connection closed.
            // TODO: Is there a better way to do this? The way this is setup, we won't
            // detect that a websocket has closed until an event came out.
            if websocket.is_closed() {
                break;
            }

            // Serialize the event as JSON and send it through the socket.
            let payload = serde_json::to_string(&event).expect("Unable to serialize event");
            websocket.send_text(&*payload).expect("Unable to send JSON payload");
        }
    });

    Ok(response)
}

#[derive(Debug, Clone)]
enum ApiMessage {
    PlayerRegistered(usize),
}

#[derive(Debug, Serialize)]
struct GameState {
    scores: HashMap<usize, usize>,
}

#[derive(Debug, Clone, Serialize)]
struct ClientConnectionMessage {
    id: usize,
}

#[derive(Debug, Clone, Serialize)]
enum HostUpdate {
    PlayerRegistered(usize),
}

#[derive(Debug, Clone, Serialize)]
enum ClientUpdate {
    PlayerRegistered(usize),
}
