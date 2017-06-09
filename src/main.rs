#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate ws;

use api::*;
use rocket::response::*;
use std::io;
use std::path::*;

mod api;

#[get("/<file..>")]
fn static_serve(file: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/").join(file))
}

fn main() {
    // Start websocket servers for broadcasting messages to host clients and player clients. The
    // resulting `Broadcaster<T>` objects are given to Rocket as managed state.
    let client_broadcaster = api::start_websocket_server::<PlayerBroadcast>("localhost:6768");
    let host_broadcaster = api::start_websocket_server::<HostBroadcast>("localhost:6769");

    // Start the main Rocket application.
    rocket::ignite()
        .mount("/", routes![static_serve])
        .mount("/api", routes![api::register_player])
        .manage(PlayerIdGenerator::new())
        .manage(host_broadcaster)
        .manage(client_broadcaster)
        .launch();
}
