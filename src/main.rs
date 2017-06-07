#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket::response::*;
use std::io;
use std::path::*;

mod api;

#[get("/<file..>")]
fn static_serve(file: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("www/").join(file))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![static_serve])
        .mount("/api", routes![api::register_player])
        .launch();
}
