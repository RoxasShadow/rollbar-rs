extern crate rollbar;
use rollbar::{Client, MessageToPayload};

fn main() {
    let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

    client.send("hai".build_payload(
            &client.build_report().with_level("info")));
}
