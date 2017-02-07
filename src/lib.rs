#[macro_use]
extern crate serde_json;
extern crate hyper;
extern crate hyper_openssl;
use std::io::Read;
use std::{panic, thread, fmt};
use std::borrow::ToOwned;
use std::sync::Arc;

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

#[derive(Default)]
pub struct Error {
    filename: String,
    line_number: u32,
    class: String,
    message: String,
    description: String
}

impl<T: fmt::Debug> From<T> for Error {
    fn from(error: T) -> Error {
        let error_message = format!("{:?}", error);
        Error {
            message: error_message.to_owned(),
            description: error_message,
            ..Default::default()
        }
    }
}

impl Error {
    pub fn from_panic(panic_info: &panic::PanicInfo) -> Error {
        let payload = panic_info.payload();
        let error_message = match payload.downcast_ref::<&str>() {
            Some(s) => *s,
            None => match payload.downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>"
            }
        };

        match panic_info.location() {
            Some(location) => {
                Error {
                    filename: location.file().to_owned(),
                    line_number: location.line().to_owned(),
                    class: String::new(),
                    message: error_message.to_string(),
                    description: error_message.to_string()
                }
            },
            None => {
                Error {
                    message: error_message.to_string(),
                    description: error_message.to_string(),
                    ..Default::default()
                }
            }
        }
    }
}

pub trait ErrorToPayload {
    fn build_payload(&self, report_builder: &ReportBuilder) -> String;
}

pub trait MessageToPayload {
    fn build_payload(&self, report_builder: &ReportBuilder) -> String;
}

impl ErrorToPayload for Error {
    fn build_payload(&self, report_builder: &ReportBuilder) -> String {
        let client = report_builder.client;
        json!({
            "access_token": client.access_token,
            "data": {
                "environment": client.environment,
                "body": {
                    "trace": {
                        "frames": [{
                            "filename": self.filename,
                            "lineno": self.line_number
                        }],
                        "exception": {
                            "class": self.class,
                            "message": self.message,
                            "description": self.description
                        }
                    }
                },
                "level": match report_builder.level {
                    Some(ref level) => level.to_string(),
                    None => Level::ERROR.to_string()
                },
                "language": "rust"
            }
        }).to_string()
    }
}

impl<'a> MessageToPayload for &'a str {
    fn build_payload(&self, report_builder: &ReportBuilder) -> String {
        let client = report_builder.client;
        json!({
            "access_token": client.access_token,
            "data": {
                "body": {
                    "environment": client.environment,
                    "message": {
                        "body": self
                    }
                },
                "level": match report_builder.level {
                    Some(ref level) => level.to_string(),
                    None => Level::ERROR.to_string()
                }
            }
        }).to_string()
    }
}

// https://rollbar.com/docs/api/items_post/
const URL: &'static str = "https://api.rollbar.com/api/1/item/";

pub struct ReportBuilder<'a> {
    client: &'a Client<'a>,
    level: Option<Level>,
    send_strategy: Option<Box<Fn(Arc<hyper::Client>, String)>>
}

impl<'a> ReportBuilder<'a> {
    pub fn report<T>(&mut self, error: T) -> &mut Self where T: Into<Error> {
        match self.send_strategy {
            Some(ref send_strategy) => {
                let http_client = self.client.http_client.clone();
                let payload = error.into().build_payload(&self);
                send_strategy(http_client, payload);
            },
            None => {
                let payload = error.into().build_payload(&self);
                self.client.send(payload);
            }
        };
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

pub struct Client<'a> {
    http_client: Arc<hyper::Client>,
    access_token: &'a str,
    environment: &'a str
}

impl<'a> Client<'a> {
    pub fn new(access_token: &'a str, environment: &'a str) -> Client<'a> {
        let ssl = hyper_openssl::OpensslClient::new().unwrap();
        let connector = hyper::net::HttpsConnector::new(ssl);

        Client {
            http_client: Arc::new(hyper::Client::with_connector(connector)),
            access_token: access_token,
            environment: environment
        }
    }

    pub fn build_report(&self) -> ReportBuilder {
        ReportBuilder {
            client: self,
            level: None,
            send_strategy: None
        }
    }

    pub fn send(&self, payload: String) {
        let http_client = self.http_client.clone();

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

    use std::panic;
    use super::{Client, Level, Error, MessageToPayload, ErrorToPayload};
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc::channel;

    // TODO: rewrite this shit
    #[test]
    fn test_build_payload_from_panic() {
        let (tx, rx) = channel();

        {
            let tx = Arc::new(Mutex::new(tx));

            let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");
            panic::set_hook(Box::new(move |panic_info| {
                let error = Error::from_panic(panic_info).build_payload(
                    client.build_report().with_level("info"));
                let error = Arc::new(Mutex::new(error));
                tx.lock().unwrap().send(error).unwrap();
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

        let     payload:          serde_json::Value = serde_json::from_str(&*error).unwrap();
        let mut expected_payload: serde_json::Value = serde_json::from_str(
            r#"{"access_token":"ACCESS_TOKEN","data":{"body":{"trace":{"exception":{"class":"","description":"attempt to divide by zero","message":"attempt to divide by zero"},"frames":[{"filename":"src/lib.rs","lineno":268}]}},"environment":"ENVIRONMENT","level":"info","language":"rust"}}"#
        ).unwrap();

        *expected_payload.get_mut("data").unwrap()
            .get_mut("body").unwrap()
            .get_mut("trace").unwrap()
            .get_mut("frames").unwrap() = payload.get("data").unwrap()
                                            .get("body").unwrap()
                                            .get("trace").unwrap()
                                            .get("frames").unwrap()
                                            .clone();

        assert_eq!(expected_payload.to_string(), payload.to_string());
    }

    #[test]
    fn test_report_match() {
        let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");

        match "笑".parse::<i32>() {
            Ok(_) => { println!("lolnope"); },
            Err(e) => {
                client.build_report()
                    .with_level(Level::ERROR)
                    .with_send_strategy(Box::new(|_, payload| {
                        assert_eq!(
                            payload,
                            r#"{"access_token":"ACCESS_TOKEN","data":{"body":{"trace":{"exception":{"class":"","description":"ParseIntError { kind: InvalidDigit }","message":"ParseIntError { kind: InvalidDigit }"},"frames":[{"filename":"","lineno":0}]}},"environment":"ENVIRONMENT","language":"rust","level":"error"}}"#
                        );
                    }))
                    .report(e);
            }
        }
    }


    #[test]
    fn test_payload_string() {
        let client = Client::new("ACCESS_TOKEN", "ENVIRONMENT");
        let report = client.build_report();
        assert_eq!(
            "hai".build_payload(&report),
            r#"{"access_token":"ACCESS_TOKEN","data":{"body":{"environment":"ENVIRONMENT","message":{"body":"hai"}},"level":"error"}}"#
        );
    }
}
