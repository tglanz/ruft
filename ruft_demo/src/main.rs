use std::io;
use std::path::{Path};
use std::fs;
use clap::Parser;
use serde::{Deserialize};

#[derive(Debug)]
enum Error {
    String(String)
}

#[derive(Debug, Deserialize)]
struct ClusterConfig {
    ports: Vec<usize>,
}

#[derive(Debug, Deserialize)]
struct RaftConfig {

}

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

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let config = read_config(args.config)?;
    println!("{:#?}", config);
    Ok(())
}
