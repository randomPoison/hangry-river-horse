#[macro_use]
extern crate rouille;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use rouille::websocket;
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
    println!("Now listening on localhost:6767");

    // Create the game state within an `Arc<Mutex>` so that it can be shared between threads.
    // TODO: There should be a more concurrency-friendly way to model this, likely built around
    // message passing. Using a mutex works, but forces unnecessary synchronization.
    let game_state = Arc::new(Mutex::new(GameState {
        scores: HashMap::new(),
    }));

    // Create a channel that allows client threads to notify the host thread that the game state
    // has updated. No actually data is passed through the channel, it's just used to notify the
    // host thread.
    let (sender, receiver) = mpsc::channel::<()>();

    let maybe_receiver = Arc::new(Mutex::new(Some(receiver)));
    let sender = Arc::new(sender);

    rouille::start_server("localhost:6767", move |request| {
        router!(request,
            (GET) (/api/host) => {
                let (response, websocket) = try_or_400!(websocket::start::<String>(&request, None));

                // Grab the receiver handle.
                // TODO: Support having multiple hosts.
                let mut receiver = maybe_receiver.lock().unwrap();
                let receiver = (&mut *receiver).take().expect("Multiple hosts not supported");

                // Create a handle to the game state for this connection.
                let game_state = game_state.clone();

                // Because of the nature of I/O in Rust, we need to spawn a separate thread for
                // each websocket.
                thread::spawn(move || {
                    // This line will block until the `response` above has been returned.
                    let mut websocket = websocket.recv().unwrap();

                    // Each time we get a message from the receiver we push new data across the
                    // websocket.
                    for _ in receiver {
                        // Serialize the current game state to JSON so we can send it to the host.
                        let state_json = {
                            // TODO: We don't need a lock on the mutex, an `RwLock` would allow the
                            // sender to read the game state without preventing concurrent reads.
                            let game_state = game_state.lock().unwrap();
                            serde_json::to_string(&*game_state).unwrap()
                        };

                        websocket.send_text(&*state_json).unwrap();
                    }
                });

                response
            },

            (GET) (/api/client) => {
                let (response, websocket) = try_or_400!(websocket::start::<String>(&request, None));

                // Create a handle to the game state for this connection.
                let game_state = game_state.clone();
                let sender = (&*sender).clone();

                // Because of the nature of I/O in Rust, we need to spawn a separate thread for
                // each websocket.
                thread::spawn(move || {
                    // This line will block until the `response` above has been returned.
                    let mut websocket = websocket.recv().unwrap();

                    // Generate an ID for the client and send it back to the client.
                    let client_id = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed);
                    let connection_message = ClientConnectionMessage {
                        id: client_id,
                    };
                    websocket.send_text(&*serde_json::to_string(&connection_message).unwrap()).unwrap();

                    // Add a score for the client to the game state.
                    {
                        let mut game_state = game_state.lock().unwrap();
                        game_state.scores.insert(client_id, 0);
                    }

                    for message in websocket {
                        match message {
                            websocket::Message::Text(payload) => {
                                println!("Raw payload: {}", payload);

                                // Try parsing the message as JSON, returning an error if the
                                // payload didn't conform to the right format.
                                let message = serde_json::from_str::<ClientMessage>(&*payload).unwrap();
                                println!("message: {:?}", message);

                                // Add to the client's current score.
                                if message.event == "feed-me" {
                                    let mut game_state = game_state.lock().unwrap();
                                    {
                                        let current_score = game_state.scores.get_mut(&client_id).unwrap();
                                        *current_score += message.amount;
                                    }

                                    println!("Game state: {:?}", &*game_state);

                                    sender.send(()).unwrap();
                                }
                            }

                            _ => {
                                panic!("The client API only supports JSON-encoded text messages");
                            }
                        }
                    }
                });

                response
            },

            _ => rouille::match_assets(&request, "./www/")
        )
    });
}

#[derive(Debug, Serialize)]
struct GameState {
    scores: HashMap<usize, usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct ClientMessage {
    event: String,
    amount: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ClientConnectionMessage {
    id: usize,
}
