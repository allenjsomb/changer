use serde::{Serialize, Deserialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct SocketConfig {
    pub pull: std::vec::Vec<PullConfig>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PullConfig {
    pub ack: bool,
    pub src: String,
    pub regex: String,
    pub dst: Option<String>
}

impl SocketConfig {
    pub fn load(file: &Path) -> SocketConfig {
        let f = std::fs::File::open(file.to_str().unwrap()).unwrap();
        let reader = std::io::BufReader::new(f);
        serde_yaml::from_reader(reader).unwrap()
    }

}