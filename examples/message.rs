#[macro_use]
extern crate rollbar;

use rollbar::{Client, MessageToPayload};

fn main() {
    let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");
    report_message!(client, "hai");
}
