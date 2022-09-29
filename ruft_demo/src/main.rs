#![allow(unused)]
#![allow(dead_code)]

extern crate ruft_core;

use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use ruft_core::prelude::*;

#[derive(Debug)]
enum Error {
    String(String),
}

#[derive(Debug, Deserialize)]
struct ClusterConfig {
    ports: Vec<usize>,
}

#[derive(Debug, Deserialize)]
struct RaftConfig {}

#[derive(Debug, Deserialize)]
struct Config {
    cluster: ClusterConfig,
    raft: RaftConfig,
}

#[derive(Parser, Debug)]
struct Cli {
    #[clap(long)]
    config: String,
}

fn read_config<P: AsRef<Path>>(config_path: P) -> Result<Config, Error> {
    match fs::File::open(config_path) {
        Err(error) => Err(Error::String(error.to_string())),
        Ok(file) => {
            let reader = io::BufReader::new(file);
            match serde_yaml::from_reader(reader) {
                Err(error) => Err(Error::String(error.to_string())),
                Ok(config) => Ok(config),
            }
        }
    }
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

#[derive(Debug)]
struct BasicBlobStorage {
    internal: HashMap<String, Vec<u8>>,
}

impl BasicBlobStorage {
    fn create() -> Self {
        Self {
            internal: Default::default(),
        }
    }
}

impl BlobStorage for BasicBlobStorage {
    fn save(&mut self, id: String, blob: Vec<u8>) -> Result<(), GenericError> {
        self.internal.insert(id, blob);
        Ok(())
    }

    fn load(&mut self, id: String) -> Result<Vec<u8>, GenericError> {
        match self.internal.remove(&id) {
            Some(buffer) => Ok(buffer),
            None => Err(GenericError::Generic("not found: todo improve".to_string())),
        }
    }
}

type ConcreteServerType = Server<BasicBlobStorage, String, BasicStateMachine>;

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let config = read_config(args.config)?;

    let mut servers: Vec<ConcreteServerType> = config.cluster.ports.into_iter().map(|port| {
        let state_machine = BasicStateMachine::create();
        let blob_storage = BasicBlobStorage::create();
        Server::create(blob_storage, state_machine).unwrap()
    }).collect();

    Ok(())
}
