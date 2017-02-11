#[macro_use]
extern crate rollbar;
extern crate backtrace;

fn main() {
    let client = rollbar::Client::new("ACCESS_TOKEN", "ENVIRONMENT");
    report_panics!(client);

    let zero = "0".parse::<i32>().unwrap(); // let's trick the lint a bit!
    let _ = 42/zero;
}
