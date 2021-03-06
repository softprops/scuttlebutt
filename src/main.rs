extern crate scuttlebutt;
use scuttlebutt::{Cluster, Events};

fn main() {
    match Cluster::new().events() {
        Ok(events) => {
            for e in events.into_iter() {
                println!("{:#?}", e)
            }
        }
        Err(e) => println!("{:#?}", e),
    }
}
