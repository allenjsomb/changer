mod socket_set;
mod processor;
mod rules;
mod args;

#[macro_use]
extern crate lazy_static;

use std;
use zmq::Socket;
use clap::{App, Arg};
use log::{debug, error, info, trace, warn, LevelFilter};
use env_logger::{Builder, Env, WriteStyle};
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use serde_yaml;
use std::time::SystemTime;
use crate::socket_set::SocketSet;
use args::Args;

const APP_NAME: &str = "Changer";

lazy_static! {
    static ref LOG_LEVELS: std::vec::Vec<&'static str> = vec!["debug", "error", "info", "trace", "warn"];
    static ref LOG_STYLES: std::vec::Vec<&'static str> = vec!["always", "auto", "never"];
}

fn init_logger(log_level: &str, log_style: &str) {
    if !LOG_LEVELS.contains(&log_level) {
        eprintln!(r#"Logging not configured because level "{}" is not valid. Try one of these {:?}."#,
                  log_level, *LOG_LEVELS);
        return;
    }

    let level = LevelFilter::from_str(log_level);
    if level.is_err() {
        eprintln!("Unable to config logger -> {:?}", level.unwrap_err());
        return;
    }

    let style: WriteStyle;
    match log_style {
        "always" => style = WriteStyle::Always,
        "never" => style = WriteStyle::Never,
        _ => style = WriteStyle::Auto
    }

    Builder::new()
        .filter_level(level.unwrap())
        .write_style(style)
        .format_level(true)
        .format_timestamp_micros()
        .init();

    info!(r#"Logger settings -> level "{}" / style "{}""#, log_level, log_style)
}

fn main() {
    let args = Args::get();

    init_logger(
        args.log_level.as_str(),
        args.log_style.as_str()
    );

    let cfg_file = std::path::Path::new(args.rules_file.as_str());
    if !cfg_file.exists() || !cfg_file.is_file() {
        error!("Config file {} does not exist.", args.rules_file);
        return;
    }

    debug!("{} running with settings -> {:?}", APP_NAME, args);

    info!("Using {} as rules file.", args.rules_file);
    let mut rules = rules::RulesConfig::load(cfg_file);
    debug!("Rules -> {:?}", rules);
    let mut last_modified: SystemTime = SystemTime::UNIX_EPOCH;
    loop {
        let metadata = std::fs::metadata(cfg_file.to_str().unwrap()).unwrap();
        if last_modified == SystemTime::UNIX_EPOCH {
            last_modified = metadata.modified().unwrap();
        }

        if last_modified != metadata.modified().unwrap() {
            info!("{} file has changed - applying changes.", args.rules_file);
            rules = rules::RulesConfig::load(cfg_file);
            debug!("Config -> {:?}", rules);
            last_modified = metadata.modified().unwrap();
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

fn start_sockets() {
    let ctx = zmq::Context::new();
    //let socket_set = SocketSet::new(&ctx);
}

fn pull(pull_sock: Socket, pub_sock: Socket) {
    let mut count = 0_usize;
    let mut dropped = 0_usize;
    loop {
        let data = pull_sock.recv_multipart(0).unwrap();
        if data.len() < 2 {
            dropped += 1;
            continue;
        }
        count += 1;
        println!(
            "{:020} Identity: {:?} Message : {}",
            count,
            std::str::from_utf8(&data[0]).unwrap(),
            std::str::from_utf8(&data[1]).unwrap()
        );
    }
}

fn sub(sub_sock: Socket) {
    let mut count = 0_usize;
    let mut dropped = 0_usize;
    loop {
        let data = sub_sock.recv_multipart(0).unwrap();
        if data.len() < 3 {
            dropped += 1;
            continue;
        }

        count += 1;
        println!(
            "{:020} Identity: {:?} Id: {} Message : {}",
            count,
            std::str::from_utf8(&data[0]).unwrap(),
            std::str::from_utf8(&data[1]).unwrap(),
            std::str::from_utf8(&data[2]).unwrap()
        );
    }
}
