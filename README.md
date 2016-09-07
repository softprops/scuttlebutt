# scuttlebutt [![Build Status](https://travis-ci.org/softprops/scuttlebutt.svg?branch=master)](https://travis-ci.org/softprops/scuttlebutt) [![Coverage Status](https://coveralls.io/repos/github/softprops/scuttlebutt/badge.svg)](https://coveralls.io/github/softprops/scuttlebutt)

> Listen in on all on the gossip going on in your [kubernetes](http://kubernetes.io/) cluster

## usage

Central to scuttlebut is a cluster. Clusters provide an interface for feeding off of kubernetes events
via a [Receiver](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html).

The default use case is to connect to a kubernetes cluster behind kubeproxy. This interface may be extended to run
outside a cluster with a set of [kubeconfig credentials](https://github.com/softprops/kubecfg) in the future.

```rust
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
```

Doug Tangren (softprops) 2016
