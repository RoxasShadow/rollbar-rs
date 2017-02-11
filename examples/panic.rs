#[macro_use]
extern crate rollbar;
extern crate backtrace;

use rollbar::{Client, ErrorToPayload};

fn main() {
    let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");
    report_panics!(client);

    let zero = "0".parse::<i32>().unwrap(); // let's trick the lint a bit!
    let _ = 42/zero;
}
