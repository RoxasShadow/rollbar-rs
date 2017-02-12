#[macro_use]
extern crate rollbar;

fn main() {
    let client = rollbar::Client::new("ACCESS_TOKEN", "ENVIRONMENT");
    report_message!(client, "hai");

    /* // `report_message!` expands to the following code:
     * client.build_report()
     *     .with_level(rollbar::Level::INFO)
     *     .from_message("hai")
     *     .send();
     * // If you want to customize the message, you might not want to use the macro.
    */

}
