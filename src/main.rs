extern crate hyper;
extern crate futures;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::collections::HashMap;
use std::io;
use serde_json::json;
use maud::html;

use futures::Stream;
use hyper::server::{Request, Response, Service};
use hyper::header::ContentType;
use hyper::{Chunk, StatusCode};
use futures::future::{ok, Future, FutureResult};

struct Microservice;

impl Service for Microservice {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<dyn Future<Item = Response, Error = hyper::Error> + Send>;

    fn call(&self, req: Request) -> Self::Future {
        match (req.method(), req.uri().path()) { 
            (&hyper::Method::Post, "/") => {
                let future = req
                    .body()
                    .concat2()
                    .and_then(parse_form)
                    .and_then(write_to_db)
                    .then(make_post_response);
                Box::new(future)
            }
            (&hyper::Method::Get,"/") => {
                let time_range = match req.query(){
                    Some(query) => parse_query(query),
                    None => Ok(TimeRange {
                        before : Option::<i64>::None,
                        after : Option::<i64>::None,
                    }),
                };
                let response = match time_range {
                    Ok(time_range) => {
                        let messages = fetch_messages(time_range); // Fetch messages based on the time range
                        make_get_response(messages)
                    },
                    Err(err) => make_error_response(&err.to_string()),
                };
                Box::new(response)
            }
            _ => Box::new(ok(
                Response::new()
                    .with_header(ContentType::plaintext())
                    .with_body("Not Found")
                    .with_status(StatusCode::NotFound),
            )),
        }
    } 

}

struct NewMessage {
    username : String,
    message : String,
}
struct TimeRange {
    before: Option<i64>,
    after: Option<i64>,
}

fn parse_query(query: &str) -> Result<TimeRange, String>{
    let args = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    let before = args.get("before")
        .map(|s| s.parse::<i64>());

    
    let after = args.get("after")
        .map(|s| s.parse::<i64>());

    if let Some(ref result) = before {
        if let Err(ref error) = *result {
            return Err(format!("Error parsing 'before': {}", error));
        }
    }

    if let Some(ref result) = after {
        if let Err(ref error) = *result {
            return Err(format!("Error parsing 'after': {}", error));
        }
    }

    Ok(TimeRange {
        before: before.and_then(|x| x.ok()),
        after: after.and_then(|x| x.ok()),
    })
}

fn fetch_messages(time_range: TimeRange) -> Option<Vec<NewMessage>> {
    Some(vec![
        NewMessage {
            username: "user1".to_string(),
            message: "Hello!".to_string(),
        },
        NewMessage {
            username: "user2".to_string(),
            message: "Hi there!".to_string(),
        },
    ])
}

fn make_get_response(messages: Option<Vec<NewMessage>>) -> FutureResult<Response, hyper::Error> {
    let response = match messages {
        Some(messages) => {
            let body = render_page(messages);
            Response::new()
                .with_header(ContentType::html())
                .with_body(body)
        }
        None => Response::new().with_status(StatusCode::InternalServerError),
    };
    debug!("Response: {:?}", response);
    futures::future::ok(response)
}

fn render_page(messages: Vec<NewMessage>) -> String {
    (html! {
        head {
            title {"microservice"}
            style {"body { font-family: monospace }"}
        }
        body {
            { ul {
                @for message in &messages {
                    li {
                        (message.username) " " (message.message)
                    }
                }
            } }
        }
    }).into_string()
}



fn parse_form(form_chunk: Chunk) -> FutureResult<NewMessage, hyper::Error> {
    let mut form = url::form_urlencoded::parse(form_chunk.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    if let Some(message) = form.remove("message") {
        if let Some(username) = form.remove("username") {
            return futures::future::ok(NewMessage { username, message });
        }
    }
    futures::future::err(hyper::Error::from(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Invalid form data",
    )))
        // In a real application, you would want to handle this error properly
        // and return a meaningful response to the client.
        // Here we just return an error for simplicity.
}

fn write_to_db(new_message: NewMessage) -> FutureResult<i64, hyper::Error> {
    let timestamp = 1625247600; // Example timestamp
    futures::future::ok(timestamp)
}

fn make_post_response(result: Result<i64, hyper::Error>) -> FutureResult<Response, hyper::Error> {
    match result {
        Ok(timestamp) => {
            let payload = json!({
                "timestamp": timestamp,
            })
            .to_string();
            let response = Response::new()
                .with_header(ContentType::json())
                .with_body(payload);
            debug!("Response: {:?}", response);
            futures::future::ok(response)
        },
        Err(err) => make_error_response(&err.to_string()),
    }
}

fn make_error_response(error: &str) -> FutureResult<Response, hyper::Error> {
    let payload = json!({
        "error": error,
    })
    .to_string();
    let response = Response::new()
        .with_header(ContentType::json())
        .with_body(payload)
        .with_status(StatusCode::InternalServerError);
    debug!("Error response: {:?}", response);
    futures::future::ok(response)
}


fn main() {
    env_logger::init();
    let address = "127.0.0.1:8080".parse().unwrap();
    let server = hyper::server::Http::new()
        .bind(&address, || Ok(Microservice {}))
        .unwrap();
    info!("Running microservice at {}", address);
    server.run().unwrap();
}
