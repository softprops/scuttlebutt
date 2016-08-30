extern crate hyper;
extern crate serde_json;

use hyper::{Client, Error as HttpError, Url};
use serde_json::StreamDeserializer;
use std::io::Read;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

include!(concat!(env!("OUT_DIR"), "/lib.rs"));

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Transport(HttpError),
}

impl From<HttpError> for Error {
    fn from(error: HttpError) -> Error {
        Error::Transport(error)
    }
}

pub struct Cluster {
    host: Url,
}

pub trait Events {
    fn events(&mut self) -> Result<Receiver<Event>>;

    fn generator<Iter>(&self, iter: Iter) -> Result<Receiver<Event>>
        where Iter: Iterator<Item = std::io::Result<u8>>,
              Iter: Send + 'static
    {
        let (tx, rx) = channel();
        let src: StreamDeserializer<Event, _> = StreamDeserializer::new(iter);
        thread::spawn(move || {
            for e in src {
                match e {
                    Ok(event) => match tx.send(event) {
                        Err(e) => {
                            println!("{:#?}", e);
                            break
                        },_ => ()
                    },
                    Err(e) => {
                        println!("{:#?}", e);
                        break
                    }
                }
            }
        });
        Ok(rx)
    }
}

impl Cluster {
    pub fn new() -> Cluster {
        Cluster { host: Url::parse("http://localhost:8001").unwrap() }
    }
}

impl Events for Cluster {
    fn events(&mut self) -> Result<Receiver<Event>> {
        let res = try!(Client::new()
            .get(self.host.join("/api/v1/events?watch=true").unwrap())
            .send());
        self.generator(res.bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::Receiver;
    use std::io::Read;
    #[test]
    fn events_generator() {
        impl Events for &'static str {
            fn events(&mut self) -> Result<Receiver<Event>> {
                self.generator(self.bytes().into_iter().map(|b| Ok(b)))
            }
        }
        let events = r#"{
            "object":{
                "apiVersion": "1",
                "count": 1,
                "firstTimestamp": "...",
                "lastTimestamp": "...",
                "kind":"POD",
                "message":"test",
                "involvedObject": {
                    "apiVersion": "1",
                    "kind": "POD",
                    "name": "test_name",
                    "namespace": "test_namespace"
                }
            },
            "type":"ADDED"
        }"#.events();
        assert!(
            events.unwrap()
                .into_iter()
                .map(|e| e.object.involved_object.namespace)
                .nth(0)
                    == Some("test_namespace".to_owned())
        )
    }
}
