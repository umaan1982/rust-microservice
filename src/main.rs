extern crate hyper;
extern crate futures;

#[macro_use]
extern crate log;
extern crate env_logger;

use hyper::server::{Request, Response, Service};

use futures::future::Future;

struct Microservice;

impl Service for Microservice {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<dyn Future<Item = Response, Error = hyper::Error> + Send>;

    fn call(&self, req: Request) -> Self::Future {
        let response = format!("Hello, {}!", req.uri());
        let res = Response::new()
            .with_header(hyper::header::ContentType::plaintext())
            .with_body(response);
        info!("Received request: {:?}", req);
        Box::new(futures::future::ok(res))
    }
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

