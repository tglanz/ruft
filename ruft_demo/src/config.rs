use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct ClusterConfig {
    pub ports: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RaftConfig {}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub cluster: ClusterConfig,
    pub raft: RaftConfig,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(config_path: P) -> Result<Config> {
        let file = fs::File::open(config_path)?;
        let reader = io::BufReader::new(file);
        return Ok(serde_yaml::from_reader(reader)?);
    }
}
