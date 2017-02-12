#[macro_use]
extern crate rollbar;
extern crate backtrace;

fn main() {
    let client = rollbar::Client::new("ACCESS_TOKEN", "ENVIRONMENT");
    report_panics!(client);

    /* // `report_panics!` expands to the following code:
     * std::panic::set_hook(Box::new(move |panic_info| {
     *     let backtrace = backtrace::Backtrace::new();
     *     client.build_report()
     *         .with_backtrace(&backtrace)
     *         .from_panic(panic_info)
     *         .send();
     * }));
     * // If you want to customize the reports, you might not want to use the macro.
    */

    let zero = "0".parse::<i32>().unwrap(); // let's trick the lint a bit!
    let _ = 42/zero;
}
