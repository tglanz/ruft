#![allow(unused)]
#![allow(dead_code)]

extern crate anyhow;
extern crate thiserror;
extern crate tokio;

extern crate ruft_core;

mod config;
mod memory_blob_storage;
mod ruft_proto;
mod node_service;

use anyhow::Result;
use clap::Parser;
use node_service::NodeService;
use serde::Deserialize;
use std::collections::HashMap;

use ruft_core::prelude::*;

use crate::{
    config::*,
    memory_blob_storage::*,
};

#[derive(Parser, Debug)]
struct Cli {
    #[clap(long, default_value = "assets/cluster.yaml")]
    config: String,
}

struct BasicStateMachine {}

impl BasicStateMachine {
    fn create() -> Self {
        Self {}
    }
}

impl StateMachine<String> for BasicStateMachine {
    fn apply(&mut self, command: String) -> Result<(), GenericError> {
        println!("applying: {}", command);
        Err(GenericError::Generic("not implemented".to_string()))
    }
}

type ConcreteBlobStorage = MemoryBlobStorage;
type ConcreteServer = Server<ConcreteBlobStorage, String, BasicStateMachine>;

fn format_localhost_address(port: usize) -> String {
    format!("[::1]:{}", port)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let config = Config::from_file(args.config)?;

    let mut join_handles = Vec::new();

    for port in config.cluster.ports {
        let server = {
            let state_machine = BasicStateMachine::create();
            let blob_storage = ConcreteBlobStorage::create();
            Server::create(blob_storage, state_machine).unwrap()
            
        };

        let join_handle = {
            let address = format_localhost_address(port);
            let task = NodeService::serve(address);
            tokio::spawn(task)
        };

        join_handles.push(join_handle);
    }

    for handle in join_handles {
        handle.await?;
    }

    Ok(())
}
