//! # Scuttlebutt
//!
//! Scuttlebutt is an interface for extending kubernetes by feeding off a stream of kubernetes
//! cluster events

#[macro_use]
extern crate log;
extern crate hyper;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use hyper::{Client, Error as HttpError, Url};
use std::env;
use std::io::{self, Read};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

// Kubernets cluster event
#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub object: Object,
    #[serde(rename = "type")]
    pub event_type: String,
}

/// A description of the event
#[derive(Serialize, Deserialize, Debug)]
pub struct Object {
    /// APIVersion defines the versioned schema of this representation of an object.
    /// Servers should convert recognized schemas to the latest internal value,
    /// and may reject unrecognized values. More info:
    /// http://releases.k8s.io/release-1.3/docs/devel/api-conventions.md#resources
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    /// The number of times this event has occurred.
    pub count: usize,
    /// The time at which the event was first recorded. (Time of server receipt is in TypeMeta.)
    #[serde(rename = "firstTimestamp")]
    pub first_timestamp: String,
    /// The time at which the most recent occurrence of this event was recorded.
    #[serde(rename = "lastTimestamp")]
    pub last_timestamp: String,
    /// The object that this event is about.
    #[serde(rename = "involvedObject")]
    pub involved_object: ObjectReference,
    /// Kind is a string value representing the REST resource this object represents.
    /// Servers may infer this from the endpoint the client submits requests to.
    /// Cannot be updated. In CamelCase. More info:
    /// http://releases.k8s.io/release-1.3/docs/devel/api-conventions.md#types-kinds
    pub kind: String,
    /// A human-readable description of the status of this operation.
    pub message: String,
    /// Standard object’s metadata. More info:
    /// http://releases.k8s.io/release-1.3/docs/devel/api-conventions.md#metadata
    pub metadata: ObjectMeta,
    /// This should be a short, machine understandable string that gives the reason for the
    /// transition into the object’s current status.
    pub reason: String,
    /// The component reporting this event. Should be a short machine understandable string.
    pub source: EventSource,
    /// Type of this event (Normal, Warning), new types could be added in the future
    #[serde(rename = "type")]
    pub object_type: String,
}

/// ObjectMeta is metadata that all persisted resources must have, which includes all
/// objects users must create.
#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectMeta {
    /// CreationTimestamp is a timestamp representing the server time when this object was
    // created. It is not guaranteed to be set in happens-before order across separate operations.
    // Clients may not set this value. It is represented in RFC3339 form and is in UTC.
    /// Populated by the system. Read-only. Null for lists. More info:
    /// http://releases.k8s.io/release-1.3/docs/devel/api-conventions.md#metadata
    #[serde(rename = "creationTimestamp")]
    pub creation_timestamp: String,
    /// DeletionTimestamp is RFC 3339 date and time at which this resource will be deleted.
    /// This field is set by the server when a graceful deletion is requested by the user,
    // and is not directly settable by a client. The resource will be deleted (no longer visible
    // from resource lists, and not reachable by name) after the time in this field. Once set,
    /// this value may not be unset or be set further into the future, although it may be shortened
    /// or the resource may be deleted prior to this time. For example, a user may request that a
    /// pod is deleted in 30 seconds. The Kubelet will react by sending a graceful termination
    /// signal to the containers in the pod. Once the resource is deleted in the API, the Kubelet
    /// will send a hard termination signal to the container. If not set, graceful deletion of
    /// the object has not been requested.
    /// Populated by the system when a graceful deletion is requested. Read-only. More info:
    /// http://releases.k8s.io/release-1.3/docs/devel/api-conventions.md#metadata
    #[serde(rename = "deletionTimestamp")]
    pub deletion_timestamp: Option<String>,
    /// Name must be unique within a namespace. Is required when creating resources, although
    /// some resources may allow a client to request the generation of an appropriate name
    /// automatically. Name is primarily intended for creation idempotence and configuration
    /// definition. Cannot be updated. More info:
    /// http://releases.k8s.io/release-1.3/docs/user-guide/identifiers.md#names
    pub name: String,
    /// Namespace defines the space within each name must be unique. An empty namespace is
    /// equivalent to the "default" namespace, but "default" is the canonical representation.
    /// Not all objects are required to be scoped to a namespace - the value of this field for
    /// those objects will be empty.
    /// Must be a DNS_LABEL. Cannot be updated. More info:
    /// http://releases.k8s.io/release-1.3/docs/user-guide/namespaces.md
    pub namespace: String,
    /// An opaque value that represents the internal version of this object that can be used
    /// by clients to determine when objects have changed. May be used for optimistic concurrency,
    /// change detection, and the watch operation on a resource or set of resources.
    /// Clients must treat these values as opaque and passed unmodified back to the server.
    /// They may only be valid for a particular resource or set of resources.
    /// Populated by the system. Read-only. Value must be treated as opaque by clients
    #[serde(rename = "resourceVersion")]
    pub resource_version: String,
    /// SelfLink is a URL representing this object. Populated by the system. Read-only.
    #[serde(rename = "selfLink")]
    pub self_link: String,
    /// UID is the unique in time and space value for this object. It is typically generated by
    /// the server on successful creation of a resource and is not allowed to change on PUT
    /// operations.
    /// Populated by the system. Read-only. More info:
    /// http://releases.k8s.io/release-1.3/docs/user-guide/identifiers.md#uids
    pub uid: String,
}

/// EventSource contains information for an event.
#[derive(Serialize, Deserialize, Debug)]
pub struct EventSource {
    /// Component from which the event is generated.
    pub component: String,
    /// Host name on which the event is generated.
    pub host: Option<String>,
}

/// ObjectReference contains enough information to let you inspect or modify the referred object.
#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectReference {
    /// API version of the referent.
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    /// Specific resourceVersion to which this reference is made, if any.
    #[serde(rename = "resourceVersion")]
    pub resource_version: String,
    /// UID of the referent. More info:
    /// http://releases.k8s.io/release-1.3/docs/user-guide/identifiers.md#uids
    pub uid: String,
    /// If referring to a piece of an object instead of an entire object,
    /// this string should contain a valid JSON/Go field access statement,
    /// such as desiredState.manifest.containers[2]. For example, if the object reference
    /// is to a container within a pod, this would take on a value like: "spec.containers{name}"
    /// (where "name" refers to the name of the container that triggered the event) or if no
    /// container name is specified "spec.containers[2]" (container with index 2 in this pod).
    /// This syntax is chosen only to have some well-defined way of referencing a part of an
    /// object.
    #[serde(rename = "fieldPath")]
    pub field_path: Option<String>,
    /// Kind of the referent. More info:
    /// http://releases.k8s.io/release-1.3/docs/devel/api-conventions.md#types-kinds
    pub kind: String,
    /// Name of the referent. More info:
    /// http://releases.k8s.io/release-1.3/docs/user-guide/identifiers.md#names
    pub name: String,
    /// Namespace of the referent. More info:
    /// http://releases.k8s.io/release-1.3/docs/user-guide/namespaces.md
    pub namespace: String,
}

pub type Result<T> = std::result::Result<T, Error>;

/// An enumeratation of potential errors
#[derive(Debug)]
pub enum Error {
    Transport(HttpError),
}

impl From<HttpError> for Error {
    fn from(error: HttpError) -> Error {
        Error::Transport(error)
    }
}

/// A cluster contains an address
/// for interacting with a kubernetes Cluster
/// of nodes
pub struct Cluster {
    host: Url,
}

/// Events provides a means for generating
/// a receiver for events
pub trait Events {
    fn events(&mut self) -> Result<Receiver<Event>>;

    fn generator<Bytes>(&self, bytes: Bytes) -> Result<Receiver<Event>>
    where
        Bytes: 'static + Iterator<Item = io::Result<u8>>,
        Bytes: Send,
    {
        let (tx, rx) = channel();
        let stream = serde_json::Deserializer::from_iter(bytes).into_iter::<Event>();
        thread::spawn(
            move || for e in stream {
                match e {
                    Ok(event) => {
                        if let Err(e) = tx.send(event) {
                            debug!("{:#?}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        debug!("{:#?}", e);
                        break;
                    }
                }
            },
        );
        Ok(rx)
    }
}

impl Cluster {
    pub fn new() -> Cluster {
        /// The API host to be used for communication. https://kubernetes.default:443
        let kubernetes_api_host: &str = &env::var("KUBERNETES_API_HOST").expect("KUBERNETES_API_HOST not set.");
        Cluster { host: Url::parse(kubernetes_api_host).unwrap() }
    }
}

impl Events for Cluster {
    fn events(&mut self) -> Result<Receiver<Event>> {
        let res = try!(
            Client::new()
                .get(self.host.join("/api/v1/events?watch=true").unwrap())
                .send()
        );
        self.generator(res.bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::Receiver;
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
                "kind":"Event",
                "message":"test",
                "involvedObject": {
                    "apiVersion": "1",
                    "resourceVersion": "2",
                    "uid":"2",
                    "kind": "POD",
                    "name": "test_name",
                    "namespace": "test_namespace"
                },
                "metadata": {
                    "creationTimestamp": "...",
                    "deletionTimestamp": "...",
                    "name": "test",
                    "namespace":"default",
                    "resourceVersion": "1",
                    "selfLink": "...",
                    "uid": "1"
                },
                "reason": "started",
                "source": {
                    "component": "test",
                    "host": "foo.com"
                },
                "type": "Normal"
            },
            "type":"ADDED"
        }"#
                .events();
        assert!(
            events
                .unwrap()
                .into_iter()
                .map(|e| e.object.involved_object.namespace)
                .nth(0) == Some("test_namespace".to_owned())
        )
    }
}
