# rollbar-rs
Exception tracking and logging from Rust to Rollbar.

This crate allows you to set a hook for the `panic`s that may happen during the runtime
of your application. When an error happens, it is automatically reported on [Rollbar](http://rollbar.com/).

Instead or aside the hook, you can also send direct notifications to Rollbar.

## Usage

### Automatic logging
Check `examples/panic.rs`. You can run it as it is soo check how the
payload looks like or adding your token to send a real report to Rollbar.

The code will set a hook for the [panic](https://doc.rust-lang.org/std/panic/fn.set_hook.html)s
that the application could raise and will report their information to Rollbar.

Run it with `$ cargo run --example panic`.

### Manual logging
Manual logging could be useful when we want to handle errors
but also notify Rollbar about them.

`examples/error.rs` shows how to deal with errors, while `examples/message.rs` is for plain text reports.

## TODO
- Make the API easier to use (macros?)
- Clean the code (logging crate?)
- Support Rollbar responses natively with proper structs
- Support more Rollbar fields in the payload
