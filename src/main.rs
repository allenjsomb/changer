mod supervisor;
mod rules;
mod args;
mod processor;

#[macro_use]
extern crate lazy_static;

use std;
use log::{debug, error, info, LevelFilter};
use env_logger::{Builder, WriteStyle};
use std::str::FromStr;
use crate::supervisor::Supervisor;
use args::Args;

const APP_NAME: &str = "Changer";

lazy_static! {
    static ref LOG_LEVELS: Vec<&'static str> = vec!["debug", "error", "info", "trace", "warn"];
    static ref LOG_STYLES: Vec<&'static str> = vec!["always", "auto", "never"];
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

    init_logger(args.log_level.as_str(), args.log_style.as_str());

    let cfg_file = std::path::Path::new(args.rules_file.as_str());
    if !cfg_file.exists() || !cfg_file.is_file() {
        error!("Config file {} does not exist.", args.rules_file);
        return;
    }

    debug!("{} running with settings -> {:?}", APP_NAME, args);
    info!("Using {} as rules file.", args.rules_file);

    let ctx = zmq::Context::new();
    let supervisor = Supervisor::new(
        &ctx,
        format!("tcp://{}:{}", args.ip, args.pull_port).as_str(),
        format!("tcp://{}:{}", args.ip, args.pub_port).as_str(),
        args.rhwn,
        args.shwm,
        args.rules_file,
    );
    debug!("Starting supervisor");
    supervisor.start();
}

