#[macro_use]
extern crate rollbar;
extern crate backtrace;

fn main() {
    let client = rollbar::Client::new("ACCESS_TOKEN", "ENVIRONMENT");

    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => { report_error!(client, e).join(); }
    }

    /* // `report_error!` expands to the following code:
     * let backtrace = backtrace::Backtrace::new();
     * let line = line!() - 2;
     *
     * client.build_report()
     *     .from_error(&e)
     *     .with_backtrace(&backtrace)
     *     .with_frame(rollbar::FrameBuilder::new()
     *                 .with_line_number(line)
     *                 .with_file_name(file!())
     *                 .build())
     *     .send()
     *     .join();
     * // If you want to customize the report, you might not want to use the macro.
    */
}
