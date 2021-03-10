use zmq;
use std::sync::{Arc, Mutex};
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use crate::processor::Processor;
use std::time::{SystemTime, Duration};
use crate::rules::RulesConfig;
use std::thread::sleep;
use crossbeam::thread;
use threadpool::ThreadPool;
use serde_json::{Value, Map};

lazy_static! {
    static ref PULL_PROCESSORS: Mutex<HashMap<String, Processor>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

pub struct Supervisor {
    pub pull_socket: Arc<Mutex<zmq::Socket>>,
    pub pub_socket: Arc<Mutex<zmq::Socket>>,
    pub sub_socket: Arc<Mutex<zmq::Socket>>,
    pub push_socket: Arc<Mutex<zmq::Socket>>,
    rules_file: String,
}

impl Supervisor {
    pub fn new(ctx: &zmq::Context,
               pull_url: &str,
               pub_url: &str,
               rhwm: i32,
               shwm: i32,
               rules_file: String)
               -> Supervisor {
        let pull_sock = Arc::new(
            Mutex::new(ctx.socket(zmq::PULL).unwrap()));
        let binding = pull_sock.lock().unwrap().bind(pull_url);
        if binding.is_err() {
            panic!("{} -> {}", pull_url, binding.unwrap_err());
        }
        let _res = pull_sock.lock().unwrap().set_rcvhwm(rhwm);

        let pub_sock = Arc::new(
            Mutex::new(ctx.socket(zmq::PUB).unwrap()));
        let binding = pub_sock.lock().unwrap().bind(pub_url);
        if binding.is_err() {
            panic!("{} -> {}", pub_url, binding.unwrap_err());
        }
        let _res = pub_sock.lock().unwrap().set_sndhwm(shwm);

        let push_sock = Arc::new(
            Mutex::new(ctx.socket(zmq::PUSH).unwrap()));
        let connecting = push_sock.lock().unwrap().connect(pull_url);
        if connecting.is_err() {
            panic!("{} -> {}", pull_url, connecting.unwrap_err());
        }
        let _res = push_sock.lock().unwrap().set_sndhwm(shwm);

        let sub_sock = Arc::new(
            Mutex::new(ctx.socket(zmq::SUB).unwrap()));
        let connecting = sub_sock.lock().unwrap().connect(pub_url);
        if connecting.is_err() {
            panic!("{} -> {}", pub_url, connecting.unwrap_err());
        }
        let _res = sub_sock.lock().unwrap().set_rcvhwm(rhwm);

        Supervisor {
            pull_socket: pull_sock,
            pub_socket: pub_sock,
            sub_socket: sub_sock,
            push_socket: push_sock,
            rules_file,
        }
    }
    pub fn start(&self) {
        self.load_rules();

        let res = thread::scope(|scope| {
            scope.spawn(|_| {
                self.check_for_rule_changes();
            });
            scope.spawn(|_| {
                self.start_pull_pub();
            });
            scope.spawn(|_| {
                self.start_sub_push();
            });
        });

        if res.is_err() {
            error!("{:?}", res.unwrap_err());
        }
    }

    pub fn check_for_rule_changes(&self) {
        let mut last_modified: SystemTime = SystemTime::UNIX_EPOCH;
        loop {
            let metadata = std::fs::metadata(self.rules_file.as_str()).unwrap();
            if last_modified == SystemTime::UNIX_EPOCH {
                last_modified = metadata.modified().unwrap();
            }

            if last_modified != metadata.modified().unwrap() {
                last_modified = metadata.modified().unwrap();
                info!("{} file has changed - applying changes.", self.rules_file);
                self.load_rules();
            }
            sleep(Duration::from_secs(10));
        }
    }

    fn load_rules(&self) {
        let rules = RulesConfig::load(self.rules_file.as_str());
        if rules.is_some() {
            let mut processors = PULL_PROCESSORS.lock().unwrap();
            let mut new_rules: Vec<String> = Vec::new();
            for rule in rules.unwrap().pull_rules {
                if rule.src.is_empty() {
                    error!("Rule -> {:?} missing src value.", rule);
                    continue;
                }
                new_rules.push(rule.src.clone());
                debug!("Creating process for rule {:?}", rule);
                let mut add_proc = false;
                let mut proc = Processor::new();
                if rule.regex.is_some() {
                    add_proc = proc.set_regex(rule.regex.unwrap());
                }
                if rule.dst.is_some() {
                    proc.set_destination(rule.dst.unwrap());
                }
                if add_proc {
                    processors.insert(rule.src, proc);
                }
            }
            let mut all_keys: Vec<String> = Vec::new();
            for key in processors.keys() {
                all_keys.push(key.to_owned());
            }

            debug!("all {:?} new {:?}.", all_keys, new_rules);
            for key in all_keys {
                if !new_rules.contains(&key) {
                    debug!("Removing rule {}.", key);
                    processors.remove(&key);
                }
            }
            debug!("{} processor rules - exist.", processors.len());
        } else {
            warn!("Could not apply rules.")
        }
    }

    /*
    Read from PULL socket and send message to PUB socket.
    This is the entry point for data into the system.
     */
    pub fn start_pull_pub(&self) {
        let mut count = 0_usize;
        let mut dropped = 0_usize;
        let pool = ThreadPool::new(num_cpus::get());
        loop {
            trace!("Waiting for message from PULL");
            let data = self.pull_socket.lock().unwrap()
                .recv_multipart(0).unwrap();

            if data.len() < 2 {
                dropped += 1;
                let mut msg: Option<String> = None;
                if 0 < data.len() {
                    msg = Some(String::from_utf8(data[0].clone()).unwrap());
                }
                trace!("{}/{} drop -> data: {:?}", count, dropped, msg);
                continue;
            }

            count += 1;
            let pub_socket = self.pub_socket.clone();
            pool.execute(move || {
                let src = String::from_utf8(data[0].clone()).unwrap();
                let mut id: Option<String> = None;
                let msg: String;
                if data.len() < 3 {
                    msg = String::from_utf8(data[1].clone()).unwrap();
                } else {
                    id = Some(String::from_utf8(data[1].clone()).unwrap());
                    msg = String::from_utf8(data[2].clone()).unwrap();
                }

                trace!("{}/{} src: {:?} id: {:?} data : {}", count, dropped, src, id, msg);
                // check for built in src
                match src.as_str() {
                    "changer.ack" => {
                        // TODO: msg is message_id -> update ACK queue.
                        trace!("Ack received for msg({})", msg);
                    }
                    &_ => {
                        let msg = vec![data[0].clone(), count.to_string().into_bytes(), data[1].clone()];
                        let res = pub_socket.lock().unwrap().send_multipart(msg, 0);
                        if res.is_err() {
                            trace!("{:?}", res.unwrap_err());
                        }
                    }
                }
            });
        }
    }

    pub fn start_sub_push(&self) {
        let mut count = 0_usize;
        let mut dropped = 0_usize;
        let pool = ThreadPool::new(num_cpus::get());
        let res = self.sub_socket.lock().unwrap().set_subscribe("".as_bytes());
        if res.is_err() {
            error!("{:?}", res.unwrap_err());
        }

        loop {
            trace!("Waiting for message from PUB");
            let data = self.sub_socket.lock().unwrap().recv_multipart(0).unwrap();
            if data.len() < 3 {
                dropped += 1;
                continue;
            }
            count += 1;
            let push_socket = self.push_socket.clone();
            pool.execute(move || {
                let src = String::from_utf8(data[0].clone()).unwrap();
                let id = String::from_utf8(data[1].clone()).unwrap();
                let msg = String::from_utf8(data[2].clone()).unwrap();
                trace!("{}/{} src: {:?} id: {} data : {}", count, dropped, src, id, msg);
                {
                    let map = PULL_PROCESSORS.lock().unwrap();
                    let proc_opt = map.get(&src);
                    if proc_opt.is_some() {
                        let proc = proc_opt.unwrap();
                        trace!("GOT a processor for '{}'", src);
                        let result = proc.apply(msg);
                        if result.is_some() {
                            let json: Value = result.unwrap();
                            trace!("RESULT -> {:?}", json);
                            if proc.dst.is_some() {
                                let dst: String = proc.dst.clone().unwrap();
                                let msg = vec![
                                    serde_json::to_vec(&dst).unwrap(),
                                    data[1].clone(),
                                    serde_json::to_vec(&json).unwrap()
                                ];
                                push_socket.lock().unwrap().send_multipart(msg, 0);
                            }
                        }
                    }
                }
            });
        }
    }
}