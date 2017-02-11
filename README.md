# rollbar-rs

[![Build Status](https://travis-ci.org/benashford/rs-es.svg?branch=master)](https://travis-ci.org/benashford/rs-es)

Exception tracking and logging from Rust to Rollbar.

This crate allows you to set a hook for the `panic`s that may happen during the runtime
of your application. When an error happens, it is automatically reported on [Rollbar](http://rollbar.com/).

Instead or aside the hook, you can also send direct notifications to Rollbar.

## Usage

### Automatic logging
Check `examples/panic.rs`. You can run it as it is to check how the
payload looks like or adding your token to send a real report to Rollbar (but remember to
replace the placeholder with an actual token before).

The code will set a hook for the [panic](https://doc.rust-lang.org/std/panic/fn.set_hook.html)s
that the application could raise and will report their information to Rollbar.

Run it with `$ cargo run --example panic`.

### Manual logging
Manual logging could be useful when we want to handle errors but also notify Rollbar about them.

`examples/error.rs` shows how to deal with errors, while `examples/message.rs` is for plain text reports.

### Advanced stuff
Check the source to check what are the macros used in the example files actually doing, and how you
can fill the reports with more informations.

## TODO
- Support Rollbar responses natively with proper structs
- Support more Rollbar fields in the payload
- Doc
