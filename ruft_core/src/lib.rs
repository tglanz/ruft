#![feature(trait_alias)]
#![allow(unused)]

mod model;
mod server;

pub mod prelude {
    pub use crate::model::Command;
    pub use crate::server::{BlobStorage, GenericError, Server, StateMachine};
}
