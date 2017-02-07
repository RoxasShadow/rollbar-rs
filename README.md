# rollbar-rs
Exception tracking and logging from Rust to Rollbar.

This crate allows you to set a hook for the `panic`s that may happen during the runtime
of your application. When an error happens, it is automatically reported on [Rollbar](http://rollbar.com/).

Instead or aside the hook, you can also send direct notifications to Rollbar.

## Usage

### Automatic logging
```rs
extern crate rollbar;

use std::panic;
use rollbar::*;

fn main() {
    let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

    panic::set_hook(Box::new(move |panic_info| {
        let error = Error::from_panic(panic_info).build_payload(
            client.build_report().with_level("info"));
        client.send(error);
    }));

    let _ = 42/0;
}
```

The code above will set a hook for the [panic](https://doc.rust-lang.org/std/panic/fn.set_hook.html)s
that the application could raise and will report their information to Rollbar.

If we replace the hook present in the code with the following one,
we'll be able to read the payload that Rollbar will receive.

```rs
    panic::set_hook(Box::new(move |panic_info| {
        let error = Error::from_panic(panic_info).build_payload(
            client.build_report().with_level("info"));
        println!("{}", error);
    }));
```


```js
{
   "access_token":"ACCESS_TOKEN",
   "data":{
      "body":{
         "trace":{
            "exception":{
               "class":"",
               "description":"attempt to divide by zero",
               "message":"attempt to divide by zero"
            },
            "frames":[
               {
                  "filename":"src/main.rs",
                  "lineno":12
               }
            ]
         }
      },
      "environment":"staging",
      "language":"rust",
      "level":"info"
   }
}
```

### Manual logging
Manual logging could be useful when we want to handle errors
but also notify Rollbar about them.

The first syntax is for errors, the second is for plain messages.

```rs
extern crate rollbar;
use rollbar::{Client, Level};

fn main() {
    let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => {
            client.build_report()
                .with_level(Level::ERROR)
                .report(e);
        }
    }

    client.send("hai".build_payload(&client.build_report()));
}
```

As before, let's replace the reporting lines with the following code
to see what Rollbar is going to receive.

```rs
    let error = Error::from(e);
    let payload = error.build_payload(&client.build_report());
    println!("{}", payload);

    // ...

    let payload = "hai".build_payload(&client.build_report());
    println!("{}", payload);
```

```js
{
   "access_token":"ACCESS_TOKEN",
   "data":{
      "body":{
         "trace":{
            "exception":{
               "class":"",
               "description":"ParseIntError { kind: InvalidDigit }",
               "message":"ParseIntError { kind: InvalidDigit }"
            },
            "frames":[
               {
                  "filename":"",
                  "lineno":0
               }
            ]
         }
      },
      "environment":"ENVIRONMENT",
      "language":"rust",
      "level":"error"
   }
}

// ...

{
   "access_token":"ACCESS_TOKEN",
   "data":{
      "body":{
         "environment":"ENVIRONMENT",
         "message":{
            "body":"hai"
         }
      },
      "language":"rust",
      "level":"error"
   }
}
```

## TODO
- Make the API easier to use (macros?)
- Clean the code
- Support Rollbar responses natively with proper structs
- Consider the use of some logging crate
- Support more Rollbar fields
