#[macro_use]
extern crate rollbar;
extern crate backtrace;

fn main() {
    let client = rollbar::Client::new("ACCESS_TOKEN", "ENVIRONMENT");

    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => { report_error!(client, e); }
    }
}
