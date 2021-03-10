use log::{debug, error, warn};
use regex::Regex;
use serde_json::map::Map;
use serde_json::{json, Value};

pub struct Processor {
    pub regex: Option<Regex>,
    pub dst: Option<String>,
    pub names: Vec<String>
}

impl Processor {
    pub fn new() -> Processor {
        Processor { regex: None, dst: None, names: vec!() }
    }

    pub fn set_regex(&mut self, regex_str: String) -> bool {
        let res = Regex::new(regex_str.clone().as_str());
        if res.is_ok() {
            let regex = res.unwrap();
            for name in regex.capture_names() {
                if name.is_some() {
                    self.names.push(String::from(name.unwrap()));
                    debug!("Regex contains name capture for {}", name.unwrap());
                }
            }
            if self.names.len() > 0 {
                self.regex = Some(regex);
                return true;
            } else {
                warn!("There are no capture names in '{}' - ignoring.", regex_str);
            }
        } else {
            error!("{:?} - could not compile regex '{}'", res.unwrap_err(), regex_str);
        }
        false
    }

    pub fn set_destination(&mut self, dst: String) {
        self.dst = Some(dst);
    }

    pub fn apply(&self, line: String) -> Option<Value> {
        if !line.is_empty() && self.regex.is_some() {
            let captures = self.regex.as_ref().unwrap().captures(line.as_str());
            if captures.is_some() {
                let cap = captures.unwrap();
                let mut map: Map<String, Value> = Map::new();
                for name in self.names.clone() {
                    let val = cap.name(name.as_str());
                    if val.is_some() {
                        map.insert(name, Value::from(val.unwrap().as_str()));
                    }
                }
                return Some(json!(map));
            }
        }
        None
    }
}