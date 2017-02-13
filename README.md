# rollbar-rs
[![Build Status](https://travis-ci.org/benashford/rs-es.svg?branch=master)](https://travis-ci.org/benashford/rs-es)
[![](https://meritbadge.herokuapp.com/rollbar)](https://crates.io/crates/rollbar)

Track and report errors, exceptions and messages from your Rust application to [Rollbar](https://rollbar.com/).

## Usage

### Automatic logging
`examples/panic.rs` will show you how to set a hook for all the
[panic](https://doc.rust-lang.org/std/panic/fn.set_hook.html)s that your application could raise
so that they will be handled automatically by `rollbar-rs` in order to be tracked on Rollbar.

You can run it with `$ cargo run --example panic` if you remember to set the correct `access_token`.

### Manual logging
Manual logging could be useful when you want to handle errors in your application but also notify Rollbar about them.

`examples/error.rs` shows how to deal with errors, while `examples/message.rs` is for plain text reports.

### Customize the reports
Check the documentation to understand how you can add or modify one or more fields in the reports that will be
sent to Rollbar. Generally, all the methods whose names starts with `with_` or `from_` is what you need.

You can easily generate the documentation locally by running `$ cargo doc` and then `$ open target/doc/rollbar/index.html`.
