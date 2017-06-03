#[macro_use] extern crate nickel;

use nickel::*;

fn main() {
    let mut server = Nickel::new();

    // TODO: This doesn't give quite the right behavior. You have to navigate to `/display/`,
    // going to `/display` gives a client response.
    server.mount("/display/", middleware! { |_request, response|
        "Got display request"
    });

    server.mount("/", middleware! { |_request, response|
        "Got client request"
    });

    server.listen("127.0.0.1:6767").unwrap();
}
