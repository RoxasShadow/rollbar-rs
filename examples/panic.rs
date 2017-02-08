extern crate rollbar;
extern crate backtrace;

use std::panic;
use rollbar::*;

fn main() {
    let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

    panic::set_hook(Box::new(move |panic_info| {
        let backtrace = backtrace::Backtrace::new();
        let error = Error::from_panic(panic_info, &backtrace).build_payload(
            client.build_report().with_level("info"));
        client.send(error);
    }));

    let zero = "0".parse::<i32>().unwrap(); // let's trick the lint a bit!
    let _ = 42/zero;
}
