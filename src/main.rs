#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate ws;

use rocket::response::*;
use std::io;
use std::path::*;

mod api;

#[get("/<file..>")]
fn static_serve(file: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/").join(file))
}

fn main() {
    // Start websocket server.
    // TODO: Do something with the sender?
    let sender = api::start_websocket_server();

    // Start the main Rocket application.
    rocket::ignite()
        .mount("/", routes![static_serve])
        .mount("/api", routes![api::register_player])
        .launch();
}
