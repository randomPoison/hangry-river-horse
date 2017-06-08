#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate ws;

use rocket::response::*;
use std::io;
use std::path::*;
use std::sync::*;

mod api;

#[get("/<file..>")]
fn static_serve(file: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/").join(file))
}

fn main() {
    // Start websocket server, getting the sender that can be used to broadcast messages to all
    // connected websockets.
    let broadcast_sender = api::start_websocket_server();

    // Start the main Rocket application.
    rocket::ignite()
        .manage(api::PlayerIdGenerator::new())
        .manage(Mutex::new(broadcast_sender))
        .mount("/", routes![static_serve])
        .mount("/api", routes![api::register_player])
        .launch();
}
