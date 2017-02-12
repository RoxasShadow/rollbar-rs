//! Track and report errors, exceptions and messages from your Rust application to Rollbar.

#[macro_use] extern crate serde_json;
extern crate hyper;
extern crate hyper_openssl;
extern crate backtrace;

use std::{thread, fmt, panic};
use std::borrow::ToOwned;
use std::sync::Arc;
use backtrace::Backtrace;

/// Report an error. Any type that implements `fmt::Debug` is accepted.
#[macro_export]
macro_rules! report_error {
    ($client:ident, $err:ident) => {
        let backtrace = backtrace::Backtrace::new();
        let line = line!() - 2;

        $client.build_report()
            .with_backtrace(&backtrace)
            .with_line_number(line)
            .with_file_name(file!())
            .from_error(&$err)
            .send();
    }
}

/// Set a global hook for the `panic`s your application could raise.
#[macro_export]
macro_rules! report_panics {
    ($client:ident) => {
        std::panic::set_hook(Box::new(move |panic_info| {
            let backtrace = backtrace::Backtrace::new();
            $client.build_report()
                .with_backtrace(&backtrace)
                .from_panic(panic_info)
                .send();
        }));
    }
}

/// Send a plain text message to Rollbar with severity level `INFO`.
#[macro_export]
macro_rules! report_message {
    ($client:ident, $message:expr) => {
        $client.build_report()
            .with_level(rollbar::Level::INFO)
            .from_message($message)
            .send();
    }
}

/// Variants for setting the severity level.
/// If not specified, the default value is `ERROR`.
#[derive(Clone)]
pub enum Level {
    CRITICAL,
    ERROR,
    WARNING,
    INFO,
    DEBUG
}

impl<'a> From<&'a str> for Level {
    fn from(s: &'a str) -> Level {
        match s {
            "critical" => Level::CRITICAL,
            "warning"  => Level::WARNING,
            "info"     => Level::INFO,
            "debug"    => Level::DEBUG,
            _          => Level::ERROR
        }
    }
}

impl ToString for Level {
    fn to_string(&self) -> String {
        match self {
            &Level::CRITICAL => "critical".to_string(),
            &Level::ERROR    => "error".to_string(),
            &Level::WARNING  => "warning".to_string(),
            &Level::INFO     => "info".to_string(),
            &Level::DEBUG    => "debug".to_string()
        }
    }
}

// https://rollbar.com/docs/api/items_post/
const URL: &'static str = "https://api.rollbar.com/api/1/item/";

/// Builder for a generic request to Rollbar.
pub struct ReportBuilder<'a> {
    client: &'a Client,
    send_strategy: Option<Box<Fn(Arc<hyper::Client>, String) -> thread::JoinHandle<Option<ResponseStatus>>>>,

    level: Option<Level>,
    backtrace: Option<&'a Backtrace>,
    line_number: Option<u32>,
    filename: Option<&'static str>
}

/// Builder specialized for reporting panics.
pub struct ReportPanicBuilder<'a> {
    report_builder: &'a ReportBuilder<'a>,
    panic_info: &'a panic::PanicInfo<'a>
}

impl<'a> ReportPanicBuilder<'a> {
    pub fn send(&mut self) -> thread::JoinHandle<Option<ResponseStatus>> {
        let client = self.report_builder.client;

        match self.report_builder.send_strategy {
            Some(ref send_strategy) => {
                let http_client = client.http_client.to_owned();
                send_strategy(http_client, self.to_string())
            },
            None => { client.send(self.to_string()) }
        }
    }
}

impl<'a> ToString for ReportPanicBuilder<'a> {
    fn to_string(&self) -> String {
        let report_builder = self.report_builder;
        let client = report_builder.client;

        let payload = self.panic_info.payload();
        let error_message = match payload.downcast_ref::<&str>() {
            Some(s) => *s,
            None => match payload.downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>"
            }
        };

        let frame = match self.panic_info.location() {
            Some(location) => {
                json!({
                    "filename": location.file().to_owned(),
                    "lineno": location.line().to_owned()
                })
            },
            None => {
                json!({
                    "filename": report_builder.filename.unwrap_or(""),
                    "lineno": report_builder.line_number.unwrap_or(0),
                })
            }
        };

        json!({
            "access_token": client.access_token,
            "data": {
                "environment": client.environment,
                "body": {
                    "trace": {
                        "frames": [frame],
                        "exception": {
                            "class": thread::current().name().unwrap_or("unnamed"),
                            "message": error_message,
                            "description": match report_builder.backtrace {
                                Some(backtrace) => format!("{:?}", backtrace),
                                None => error_message.to_owned()
                            }
                        }
                    }
                },
                "level": report_builder.level
                    .to_owned()
                    .unwrap_or(Level::ERROR)
                    .to_string(),
                "language": "rust"
            }
        }).to_string()
    }
}

/// Builder specialized for reporting errors.
pub struct ReportErrorBuilder<'a, T: 'a + fmt::Debug> {
    report_builder: &'a ReportBuilder<'a>,
    error: &'a T
}

impl<'a, T: fmt::Debug> ReportErrorBuilder<'a, T> {
    pub fn send(&mut self) -> thread::JoinHandle<Option<ResponseStatus>> {
        let client = self.report_builder.client;

        match self.report_builder.send_strategy {
            Some(ref send_strategy) => {
                let http_client = client.http_client.to_owned();
                send_strategy(http_client, self.to_string())
            },
            None => { client.send(self.to_string()) }
        }
    }
}

impl<'a, T: fmt::Debug> ToString for ReportErrorBuilder<'a, T> {
    fn to_string(&self) -> String {
        let report_builder = self.report_builder;
        let client = report_builder.client;
        let error_message = format!("{:?}", self.error);

        json!({
            "access_token": client.access_token,
            "data": {
                "environment": client.environment,
                "body": {
                    "trace": {
                        "frames": [{
                            "filename": report_builder.filename.unwrap_or(""),
                            "lineno": report_builder.line_number.unwrap_or(0)
                        }],
                        "exception": {
                            "class": thread::current().name().unwrap_or("unnamed"),
                            "message": error_message,
                            "description": match report_builder.backtrace {
                                Some(backtrace) => format!("{:?}", backtrace),
                                None => error_message.to_owned()
                            }
                        }
                    }
                },
                "level": report_builder.level
                    .to_owned()
                    .unwrap_or(Level::ERROR)
                    .to_string(),
                "language": "rust"
            }
        }).to_string()
    }
}

/// Builder specialized for reporting messages.
pub struct ReportMessageBuilder<'a> {
    report_builder: &'a ReportBuilder<'a>,
    message: &'a str
}

impl<'a> ReportMessageBuilder<'a> {
    pub fn send(&mut self) -> thread::JoinHandle<Option<ResponseStatus>> {
        let client = self.report_builder.client;

        match self.report_builder.send_strategy {
            Some(ref send_strategy) => {
                let http_client = client.http_client.to_owned();
                send_strategy(http_client, self.to_string())
            },
            None => { client.send(self.to_string()) }
        }
    }
}

impl<'a> ToString for ReportMessageBuilder<'a> {
    fn to_string(&self) -> String {
        let report_builder = self.report_builder;
        let client = report_builder.client;

        json!({
            "access_token": client.access_token,
            "data": {
                "environment": client.environment,
                "body": {
                    "message": {
                        "body": self.message
                    }
                },
                "level": report_builder.level
                    .to_owned()
                    .unwrap_or(Level::INFO)
                    .to_string()
            }
        }).to_string()
    }
}

impl<'a> ReportBuilder<'a> {
    /// To be used when a panic report must be sent.
    pub fn from_panic(&'a mut self, panic_info: &'a panic::PanicInfo) -> ReportPanicBuilder<'a> {
        ReportPanicBuilder {
            report_builder: self,
            panic_info: panic_info
        }
    }

    /// To be used when a error must be reported.
    /// Any type that implements `fmt::Debug` is accepted.
    pub fn from_error<T: fmt::Debug>(&'a mut self, error: &'a T) -> ReportErrorBuilder<'a, T> {
        ReportErrorBuilder {
            report_builder: self,
            error: error
        }
    }

    /// To be used when a message must be tracked by Rollbar.
    pub fn from_message(&'a mut self, message: &'a str) -> ReportMessageBuilder<'a> {
        ReportMessageBuilder {
            report_builder: self,
            message: message
        }
    }

    /// Attach a `backtrace::Backtrace` to the `description` of the report.
    pub fn with_backtrace(&mut self, backtrace: &'a Backtrace) -> &mut Self {
        self.backtrace = Some(backtrace);
        self
    }

    /// Set the number of the line in which an error occurred.
    pub fn with_line_number(&mut self, line_number: u32) -> &mut Self {
        self.line_number = Some(line_number);
        self
    }

    /// Tell the origin of the error by adding the file name to the report.
    pub fn with_file_name(&mut self, filename: &'static str) -> &mut Self {
        self.filename = Some(filename);
        self
    }

    /// Set the security level of the report. `Level::ERROR` is the default value
    pub fn with_level<T>(&'a mut self, level: T) -> &'a mut Self where T: Into<Level> {
        self.level = Some(level.into());
        self
    }

    /// Use given function to send a request to Rollbar instead of the built-in one.
    pub fn with_send_strategy(&'a mut self, send_strategy: Box<Fn(Arc<hyper::Client>, String) -> thread::JoinHandle<Option<ResponseStatus>>>) -> &'a mut Self {
        self.send_strategy = Some(send_strategy);
        self
    }
}

/// The access point to the library.
pub struct Client {
    http_client: Arc<hyper::Client>,
    access_token: String,
    environment: String
}

impl Client {
    /// Create a new `Client`.
    ///
    /// Your available `environment`s are listed at
    /// <https://rollbar.com/{your_organization}/{your_app}/settings/general>.
    ///
    /// You can get the `access_token` at
    /// <https://rollbar.com/{your_organization}/{your_app}/settings/access_tokens>.
    pub fn new<T: Into<String>>(access_token: T, environment: T) -> Client {
        let ssl = hyper_openssl::OpensslClient::new().unwrap();
        let connector = hyper::net::HttpsConnector::new(ssl);

        Client {
            http_client: Arc::new(hyper::Client::with_connector(connector)),
            access_token: access_token.into(),
            environment: environment.into()
        }
    }

    /// Create a `ReportBuilder` to build a new report for Rollbar.
    pub fn build_report(&self) -> ReportBuilder {
        ReportBuilder {
            client: self,
            send_strategy: None,
            level: None,
            backtrace: None,
            line_number: None,
            filename: None
        }
    }

    /// Function used internally to send payloads to Rollbar as default `send_strategy`.
    fn send(&self, payload: String) -> thread::JoinHandle<Option<ResponseStatus>> {
        let http_client = self.http_client.to_owned();

        thread::spawn(move || {
            let res = http_client.post(URL).body(&*payload).send();

            match res {
                Ok(res) => {
                    let status: ResponseStatus = res.status.into();

                    if status.0 != hyper::status::StatusCode::Ok {
                        print!("Your application raised an error:\n{}\n\n", payload);

                        println!("Error while sending a report to Rollbar.");
                        print!("The error returned by Rollbar was: {}.\n\n", status.to_string());
                    }

                    Some(status)
                },
                Err(err) => {
                    print!("Your application raised an error:\n{}\n\n", payload);

                    println!("Error while sending a report to Rollbar.");
                    print!("The error returned by Rollbar was: {:?}.\n\n", err);

                    None
                }
            }
        })
    }
}

/// Wrapper for `hyper::status::StatusCode`.
#[derive(Debug)]
pub struct ResponseStatus(hyper::status::StatusCode);

impl From<hyper::status::StatusCode> for ResponseStatus {
    fn from(status_code: hyper::status::StatusCode) -> ResponseStatus {
        ResponseStatus(status_code)
    }
}

impl ResponseStatus {
    /// Return a description provided by Rollbar for the status code returned by each request.
    pub fn description(&self) -> &str {
        match self.0.to_u16() {
            200 => "The item was accepted for processing.",
            400 => "No JSON payload was found, or it could not be decoded.",
            401 => "No access token was found in the request.",
            403 => "Check that your `access_token` is valid, enabled, and has the correct scope. The response will contain a `message` key explaining the problem.",
            413 => "Max payload size is 128kb. Try removing or truncating unnecessary large data included in the payload, like whole binary files or long strings.",
            422 => "A syntactically valid JSON payload was found, but it had one or more semantic errors. The response will contain a `message` key describing the errors.",
            429 => "Request dropped because the rate limit has been reached for this access token, or the account is on the Free plan and the plan limit has been reached.",
            500 => "There was an error on Rollbar's end",
            _   => "An undefined error occurred."
        }
    }

    /// Return the canonical description for the status code returned by each request.
    pub fn canonical_reason(&self) -> String {
        format!("{}", self.0)
    }
}

impl fmt::Display for ResponseStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {}: {}", self.canonical_reason(), self.description())
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    extern crate hyper;
    extern crate backtrace;

    use std::panic;
    use super::{Client, Level};
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc::channel;
    use backtrace::Backtrace;
    use serde_json::Value;

    #[test]
    fn test_report_panics() {
        let (tx, rx) = channel();

        {
            let tx = Arc::new(Mutex::new(tx));

            let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");
            panic::set_hook(Box::new(move |panic_info| {
                let backtrace = Backtrace::new();
                let payload = client.build_report()
                    .with_backtrace(&backtrace)
                    .with_level("info")
                    .from_panic(panic_info)
                    .to_string();
                let payload = Arc::new(Mutex::new(payload));
                tx.lock().unwrap().send(payload).unwrap();
            }));

            let result = panic::catch_unwind(|| {
                // just to trick the linter
                let zero = "0".parse::<i32>().unwrap();
                let _ = 1/zero;
            });
            assert!(result.is_err());
        }

        // remove the hook to avoid double panics
        let _ = panic::take_hook();

        let lock = rx.recv().unwrap();
        let payload = match lock.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let payload: Value = serde_json::from_str(&*payload).unwrap();
        let mut expected_payload = json!({
            "access_token": "ACCESS_TOKEN",
            "data": {
                "environment": "ENVIRONMENT",
                "body": {
                    "trace": {
                        "frames": [{
                            "filename": "src/lib.rs",
                            "lineno": 268
                        }],
                        "exception": {
                            "class": "tests::test_report_panics",
                            "message": "attempt to divide by zero",
                            "description": "attempt to divide by zero"
                        }
                    }
                },
                "level": "info",
                "language": "rust"
            }
        });

        // copy the frames from the payload
        *expected_payload.get_mut("data").unwrap()
            .get_mut("body").unwrap()
            .get_mut("trace").unwrap()
            .get_mut("frames").unwrap() = payload.get("data").unwrap()
                                            .get("body").unwrap()
                                            .get("trace").unwrap()
                                            .get("frames").unwrap()
                                            .to_owned();

        // copy the backtrace from the payload
        *expected_payload.get_mut("data").unwrap()
            .get_mut("body").unwrap()
            .get_mut("trace").unwrap()
            .get_mut("exception").unwrap()
            .get_mut("description").unwrap() = payload.get("data").unwrap()
                                                .get("body").unwrap()
                                                .get("trace").unwrap()
                                                .get("exception").unwrap()
                                                .get("description").unwrap()
                                                .to_owned();

        assert_eq!(expected_payload.to_string(), payload.to_string());
    }

    #[test]
    fn test_report_error() {
        let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

        match "笑".parse::<i32>() {
            Ok(_) => { assert!(false); },
            Err(e) => {
                let payload = client.build_report()
                    .with_level(Level::ERROR)
                    .from_error(&e)
                    .to_string();

                let expected_payload = json!({
                    "access_token": "ACCESS_TOKEN",
                    "data": {
                        "environment": "ENVIRONMENT",
                        "body": {
                            "trace": {
                                "frames": [{
                                    "filename": "",
                                    "lineno": 0
                                }],
                                "exception": {
                                    "class": "tests::test_report_error",
                                    "message": "ParseIntError { kind: InvalidDigit }",
                                    "description": "ParseIntError { kind: InvalidDigit }"
                                }
                            }
                        },
                        "level": "error",
                        "language": "rust"
                    }
                }).to_string();

                assert_eq!(payload, expected_payload);
            }
        }
    }

    #[test]
    fn test_report_message() {
        let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

        let payload = client.build_report()
            .with_level("info")
            .from_message("hai")
            .to_string();

        let expected_payload = json!({
            "access_token": "ACCESS_TOKEN",
            "data": {
                "environment": "ENVIRONMENT",
                "body": {
                    "message": {
                        "body": "hai"
                    }
                },
                "level": "info"
            }
        }).to_string();

        assert_eq!(payload, expected_payload);
    }

    #[test]
    fn test_response() {
        let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

        let status_handle = client.build_report()
            .with_level("info")
            .from_message("hai")
            .send();

        match status_handle.join().unwrap() {
            Some(status) => {
                assert_eq!(status.to_string(),
                    "Error 401 Unauthorized: No access token was found in the request.".to_owned());
            },
            None => { assert!(false); }
        }
    }
}
