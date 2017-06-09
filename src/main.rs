#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate ws;

use broadcast::*;
use game::*;
use rocket::response::*;
use std::io;
use std::path::*;
use std::sync::*;

mod api;
mod broadcast;
mod game;

#[get("/")]
fn static_serve_player() -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/client.html"))
}

#[get("/display")]
fn static_serve_display() -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/host.html"))
}

/// Serves files from the `www/` directory.
#[get("/<file..>")]
fn static_serve(file: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/").join(file))
}

fn main() {
    // Start websocket servers for broadcasting messages to host clients and player clients. The
    // resulting `Broadcaster<T>` objects are given to Rocket as managed state so that any API
    // endpoint can broadcast state changes as necessary.
    let client_broadcaster = broadcast::start_server::<PlayerBroadcast>("localhost:6768");
    let host_broadcaster = broadcast::start_server::<HostBroadcast>("localhost:6769");

    // Start the main Rocket application.
    rocket::ignite()
        .mount("/", routes![static_serve, static_serve_player, static_serve_display])
        .mount("/api", routes![api::register_player, api::feed_player])
        .manage(PlayerIdGenerator::new())
        .manage(host_broadcaster)
        .manage(client_broadcaster)
        .manage(Mutex::new(Scoreboard::new()))
        .launch();
}
