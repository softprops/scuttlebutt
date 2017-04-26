# scuttlebutt [![Build Status](https://travis-ci.org/softprops/scuttlebutt.svg?branch=master)](https://travis-ci.org/softprops/scuttlebutt) [![Coverage Status](https://coveralls.io/repos/github/softprops/scuttlebutt/badge.svg)](https://coveralls.io/github/softprops/scuttlebutt) [![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE) [![crates.io](http://meritbadge.herokuapp.com/scuttlebutt)](https://crates.io/crates/scuttlebutt)


> Listen in on all the gossip going on in your [kubernetes](http://kubernetes.io/) cluster

[Documentation](https://softprops.github.io/scuttlebutt)

## install

Add the following to your Cargo.toml file

```toml
[dependencies]
scuttlebutt = "0.2"
```

## usage

Central to scuttlebutt is a cluster. Clusters provide an interface for feeding off of kubernetes events
via a [Receiver](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html).

The default use case is to connect to a kubernetes cluster behind [kube-proxy](http://kubernetes.io/docs/admin/kube-proxy/). This interface may be extended to run
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

Doug Tangren (softprops) 2016-2017
