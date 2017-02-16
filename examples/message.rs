#[macro_use]
extern crate rollbar;

fn main() {
    let client = rollbar::Client::new("ACCESS_TOKEN", "ENVIRONMENT");
    let _ = report_message!(client, "hai").join();

    /* // `report_message!` expands to the following code:
     * client.build_report()
     *     .from_message("hai")
     *     .with_level(rollbar::Level::INFO)
     *     .send();
     * // If you want to customize the message, you might not want to use the macro.
     * // Join the thread only for testing purposes.
    */
}
