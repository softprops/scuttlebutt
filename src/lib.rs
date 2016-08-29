extern crate hyper;
extern crate serde_json;

use hyper::{Client, Error as HttpError};
use serde_json::StreamDeserializer;
 use std::io::Read;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

include!(concat!(env!("OUT_DIR"), "/main.rs"));

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
    host: String,
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
        Cluster { host: "http://localhost:8001".to_owned() }
    }
}

impl Events for Cluster {
    fn events(&mut self) -> Result<Receiver<Event>> {
        let res = try!(Client::new()
            .get(&format!("{}/api/v1/events?watch=true", self.host))
            .send());
        self.generator(res.bytes())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
