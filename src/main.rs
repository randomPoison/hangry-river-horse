#[macro_use] extern crate nickel;

use nickel::*;

fn main() {
    let mut server = Nickel::new();

    // TODO: This doesn't give quite the right behavior. You have to navigate to `/display/`,
    // going to `/display` gives a client response.
    server.mount("/display/", middleware! { |_request, response|
        "Got display request"
    });

    server.mount("/events/", middleware! { |_request, response|
        "Here are some events"
    });

    server.mount("/", StaticFilesHandler::new("www/"));

    server.utilize(logger_fn);

    server.listen("127.0.0.1:6767").unwrap();
}

fn logger_fn<'mw>(req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
    println!("logging request from logger fn: {:?}", req.origin.uri);
    res.next_middleware()
}
