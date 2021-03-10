use serde::{Serialize, Deserialize};
use log::{debug, error};
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize, Debug)]
pub struct RulesConfig {
    pub pull_rules: Vec<PullRule>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PullRule {
    pub src: String,
    pub regex: Option<String>,
    pub dst: Option<String>,
}

impl RulesConfig {
    pub fn load(file: &str) -> Option<RulesConfig> {
        let f = File::open(file).unwrap();
        let reader = BufReader::new(f);
        let result: Result<RulesConfig, serde_yaml::Error> = serde_yaml::from_reader(reader);
        if result.is_err() {
            error!("Could not read rules file -> {}", result.unwrap_err());
            return None;
        }
        let rules = result.unwrap();
        for rule in &rules.pull_rules {
            debug!("Rule -> {:?}", rule);
        }
        return Some(rules);
    }
}