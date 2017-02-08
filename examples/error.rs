extern crate rollbar;
extern crate backtrace;

use rollbar::{Client, Level};
use backtrace::Backtrace;

fn main() {
    let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => {
            client.build_report()
                .with_level(Level::ERROR)
                .report(e, &Backtrace::new());
        }
    }
}
