#[macro_use]
extern crate rouille;

use rouille::websocket;
use std::thread;

fn main() {
    println!("Now listening on localhost:6767");

    rouille::start_server("localhost:6767", move |request| {
        router!(request,
            (GET) (/api/host) => {
                let (response, websocket) = try_or_400!(websocket::start::<String>(&request, None));

                // Because of the nature of I/O in Rust, we need to spawn a separate thread for
                // each websocket.
                thread::spawn(move || {
                    // This line will block until the `response` above has been returned.
                    let mut websocket = websocket.recv().unwrap();

                    websocket.send_text("Here's some straight up garbo").unwrap();

                    loop {
                        // We wait for a new message to come from the websocket.
                        let message = match websocket.next() {
                            Some(m) => m,
                            None => break,
                        };

                        match message {
                            websocket::Message::Text(txt) => {
                                // If the message is text, send it back with `send_text`.
                                println!("received {:?} from a websocket", txt);
                                websocket.send_text(&txt).unwrap();
                            },
                            websocket::Message::Binary(_) => {
                                println!("received binary from a websocket");
                            },
                        }
                    }
                });

                response
            },

            (GET) (/api/client) => {
                let (response, websocket) = try_or_400!(websocket::start::<String>(&request, None));

                // Because of the nature of I/O in Rust, we need to spawn a separate thread for
                // each websocket.
                thread::spawn(move || {
                    // This line will block until the `response` above has been returned.
                    let mut websocket = websocket.recv().unwrap();

                    websocket.send_text("Here's some straight up garbo").unwrap();

                    loop {
                        // We wait for a new message to come from the websocket.
                        let message = match websocket.next() {
                            Some(m) => m,
                            None => break,
                        };

                        match message {
                            websocket::Message::Text(txt) => {
                                // If the message is text, send it back with `send_text`.
                                println!("received {:?} from a websocket", txt);
                                websocket.send_text(&txt).unwrap();
                            },
                            websocket::Message::Binary(_) => {
                                println!("received binary from a websocket");
                            },
                        }
                    }
                });

                response
            },

            _ => rouille::match_assets(&request, "./www/")
        )
    });
}
