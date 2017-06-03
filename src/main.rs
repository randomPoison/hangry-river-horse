extern crate iron;
#[macro_use]
extern crate mime;
extern crate mount;
extern crate staticfile;

use iron::prelude::*;
use iron::status;
use staticfile::Static;
use mount::Mount;

fn main() {
    let mut mount = Mount::new();

    // Going to the root address should display the client-facing site.
    mount.mount("/", Static::new("www/"));

    // The `events/` endpoint should provide a stream of events to the display client.
    mount.mount("display/events/", |_request: &mut Request| {

        Ok(Response::with((
            mime!(Text/EventStream),
            status::Ok,
            "event: garbo\ndata: Here's an event\n\n",
        )))
    });

    // Instantiate and run the server.
    Iron::new(mount).http("localhost:6767").unwrap();
}
