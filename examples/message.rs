#[macro_use]
extern crate rollbar;

fn main() {
    let client = rollbar::Client::new("ACCESS_TOKEN", "ENVIRONMENT");
    report_message!(client, "hai");
}
