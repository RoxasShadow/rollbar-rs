#[macro_use]
extern crate serde_json;
extern crate hyper;
extern crate hyper_openssl;
extern crate backtrace;

use std::thread;
use std::io::Read;
use std::fmt::Debug;
use std::panic::PanicInfo;
use std::borrow::ToOwned;
use std::sync::Arc;
use backtrace::Backtrace;

#[macro_export]
macro_rules! report_error {
    ($client:ident, $err:ident) => {
        let backtrace = backtrace::Backtrace::new();
        $client.build_report()
            .with_backtrace(&backtrace)
            .with_line_number(line!())
            .with_file_name(file!())
            .from_error(&$err)
            .send();
    }
}

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

#[macro_export]
macro_rules! report_message {
    ($client:ident, $message:expr) => {
        $client.build_report()
            .with_level(rollbar::Level::INFO)
            .from_message($message)
            .send();
    }
}

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

pub struct ReportBuilder<'a> {
    client: &'a Client,
    send_strategy: Option<Box<Fn(Arc<hyper::Client>, String)>>,

    level: Option<Level>,
    backtrace: Option<&'a Backtrace>,
    line_number: Option<u32>,
    filename: Option<&'static str>
}

pub struct ReportPanicBuilder<'a> {
    report_builder: &'a ReportBuilder<'a>,
    panic_info: &'a PanicInfo<'a>
}

impl<'a> ReportPanicBuilder<'a> {
    pub fn send(&mut self) {
        let client = self.report_builder.client;

        match self.report_builder.send_strategy {
            Some(ref send_strategy) => {
                let http_client = client.http_client.to_owned();
                send_strategy(http_client, self.to_string());
            },
            None => { client.send(self.to_string()); }
        };
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

pub struct ReportErrorBuilder<'a, T: 'a + Debug> {
    report_builder: &'a ReportBuilder<'a>,
    error: &'a T
}

impl<'a, T: Debug> ReportErrorBuilder<'a, T> {
    pub fn send(&mut self) {
        let client = self.report_builder.client;

        match self.report_builder.send_strategy {
            Some(ref send_strategy) => {
                let http_client = client.http_client.to_owned();
                send_strategy(http_client, self.to_string());
            },
            None => { client.send(self.to_string()); }
        };
    }
}

impl<'a, T: Debug> ToString for ReportErrorBuilder<'a, T> {
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

pub struct ReportMessageBuilder<'a> {
    report_builder: &'a ReportBuilder<'a>,
    message: &'a str
}

impl<'a> ReportMessageBuilder<'a> {
    pub fn send(&mut self) {
        let client = self.report_builder.client;

        match self.report_builder.send_strategy {
            Some(ref send_strategy) => {
                let http_client = client.http_client.to_owned();
                send_strategy(http_client, self.to_string());
            },
            None => { client.send(self.to_string()); }
        };
    }
}

impl<'a> ToString for ReportMessageBuilder<'a> {
    fn to_string(&self) -> String {
        let report_builder = self.report_builder;
        let client = report_builder.client;

        json!({
            "access_token": client.access_token,
            "data": {
                "body": {
                    "environment": client.environment,
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
    pub fn from_panic(&'a mut self, panic_info: &'a PanicInfo) -> ReportPanicBuilder<'a> {
        ReportPanicBuilder {
            report_builder: self,
            panic_info: panic_info
        }
    }

    pub fn from_error<T: Debug>(&'a mut self, error: &'a T) -> ReportErrorBuilder<'a, T> {
        ReportErrorBuilder {
            report_builder: self,
            error: error
        }
    }

    pub fn from_message(&'a mut self, message: &'a str) -> ReportMessageBuilder<'a> {
        ReportMessageBuilder {
            report_builder: self,
            message: message
        }
    }

    pub fn with_backtrace(&mut self, backtrace: &'a Backtrace) -> &mut Self {
        self.backtrace = Some(backtrace);
        self
    }

    pub fn with_line_number(&mut self, line_number: u32) -> &mut Self {
        self.line_number = Some(line_number);
        self
    }

    pub fn with_file_name(&mut self, filename: &'static str) -> &mut Self {
        self.filename = Some(filename);
        self
    }

    pub fn with_level<T>(&'a mut self, level: T) -> &'a mut Self where T: Into<Level> {
        self.level = Some(level.into());
        self
    }

    pub fn with_send_strategy(&'a mut self, send_strategy: Box<Fn(Arc<hyper::Client>, String)>) -> &'a mut Self {
        self.send_strategy = Some(send_strategy);
        self
    }
}

pub struct Client {
    http_client: Arc<hyper::Client>,
    access_token: String,
    environment: String
}

impl Client {
    pub fn new<T: Into<String>>(access_token: T, environment: T) -> Client {
        let ssl = hyper_openssl::OpensslClient::new().unwrap();
        let connector = hyper::net::HttpsConnector::new(ssl);

        Client {
            http_client: Arc::new(hyper::Client::with_connector(connector)),
            access_token: access_token.into(),
            environment: environment.into()
        }
    }

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

    pub fn send(&self, payload: String) {
        let http_client = self.http_client.to_owned();

        let _ = thread::spawn(move || {
            let res = http_client.post(URL).body(&*payload).send();

            match res {
                Ok(mut res) => {
                    let mut body = String::new();
                    res.read_to_string(&mut body).unwrap();

                    println!("- Error while sending a report to Rollbar.");
                    println!("\n- The error that Rollbar raised was:\n{:?}", res);
                    println!("\n- The message that Rollbar returned was:\n{}", body);
                    println!("\n- The error that your application raised was:\n{}", payload);
                },
                Err(e) => {
                    println!("- Error while sending a report to Rollbar.");
                    println!("\n- The error that Rollbar raised was:\n{:?}", e);
                    println!("\n- The error that your application raised was:\n{}", payload);
                }
            }
        }).join();
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

    // TODO: rewrite this shit
    #[test]
    fn test_build_payload_from_panic() {
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
        let error = match lock.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let mut payload:          serde_json::Value = serde_json::from_str(&*error).unwrap();
        let mut expected_payload: serde_json::Value = serde_json::from_str(
            r#"{"access_token":"ACCESS_TOKEN","data":{"body":{"trace":{"exception":{"class":"tests::test_build_payload_from_panic","description":"attempt to divide by zero","message":"attempt to divide by zero"},"frames":[{"filename":"src/lib.rs","lineno":268}]}},"environment":"ENVIRONMENT","level":"info","language":"rust"}}"#
        ).unwrap();

        *expected_payload.get_mut("data").unwrap()
            .get_mut("body").unwrap()
            .get_mut("trace").unwrap()
            .get_mut("frames").unwrap() = payload.get("data").unwrap()
                                            .get("body").unwrap()
                                            .get("trace").unwrap()
                                            .get("frames").unwrap()
                                            .clone();

        // we're gonna ignore ignore the backtrace
        *payload.get_mut("data").unwrap()
            .get_mut("body").unwrap()
            .get_mut("trace").unwrap()
            .get_mut("exception").unwrap()
            .get_mut("description").unwrap() = Value::String("attempt to divide by zero".into());

        assert_eq!(expected_payload.to_string(), payload.to_string());
    }

    #[test]
    fn test_report_error() {
        let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

        match "笑".parse::<i32>() {
            Ok(_) => { println!("lolnope"); },
            Err(e) => {
                let backtrace = Backtrace::new();

                client.build_report()
                    .with_level(Level::ERROR)
                    .with_send_strategy(Box::new(|_, payload| {
                        // we're gonna ignore the backtrace
                        let mut payload: serde_json::Value = serde_json::from_str(&*payload).unwrap();
                        let     expected_payload: serde_json::Value = serde_json::from_str(
                            r#"{"access_token":"ACCESS_TOKEN","data":{"body":{"trace":{"exception":{"class":"tests::test_report_error","description":"ParseIntError { kind: InvalidDigit }","message":"ParseIntError { kind: InvalidDigit }"},"frames":[{"filename":"","lineno":0}]}},"environment":"ENVIRONMENT","language":"rust","level":"error"}}"#
                        ).unwrap();
                        *payload.get_mut("data").unwrap()
                            .get_mut("body").unwrap()
                            .get_mut("trace").unwrap()
                            .get_mut("exception").unwrap()
                            .get_mut("description").unwrap() = Value::String("ParseIntError { kind: InvalidDigit }".into());

                        assert_eq!(expected_payload.to_string(), payload.to_string());
                    }))
                    .with_backtrace(&backtrace)
                    .from_error(&e)
                    .send();
            }
        }
    }

    #[test]
    fn test_payload_string() {
        let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");
        let payload = client.build_report()
            .with_level("info")
            .from_message("hai")
            .to_string();

        assert_eq!(
            payload,
            r#"{"access_token":"ACCESS_TOKEN","data":{"body":{"environment":"ENVIRONMENT","message":{"body":"hai"}},"level":"info"}}"#
        );
    }
}
