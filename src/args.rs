use clap::{App, Arg};

#[derive(Debug)]
pub struct Args {
    pub rules_file: String,
    pub log_level: String,
    pub log_style: String,
    pub ip: String,
    pub pull_port: i32,
    pub pub_port: i32,
    pub rhwn: i32,
    pub shwm: i32,
}

impl Args {
    fn is_good_port_number(s: String) -> Result<(), String> {
        let num: i32 = s.parse::<i32>().unwrap_or(0);
        if num < 1024 || (u16::MAX as i32) < num {
            return Err(
                format!("Bad port number {}. Chose a number in this range {}-{}",
                        num, 1024, u16::MAX));
        }
        Ok(())
    }

    fn hwm_is_sane(s: String) -> Result<(), String> {
        let num: i32 = s.parse::<i32>().unwrap_or(0);
        if num < 1 {
            return Err(format!("HWM {} too low.", s));
        }
        Ok(())
    }

    pub fn get() -> Args {
        let app_name = String::from(crate::APP_NAME);
        let args = App::new(app_name.clone())
            .version("1.0")
            .author("Jimmy Allen. <allenjsomb@gmail.com>")
            .about("Data processing and transformation server.")
            .arg(Arg::with_name("rules")
                .short("r")
                .long("rules")
                .value_name("FILE")
                .default_value("./rules.yml")
                .empty_values(false)
                .help("Sets a config file")
                .takes_value(true))
            .arg(Arg::with_name("log_level")
                .short("l")
                .long("log_level")
                .default_value("info")
                .empty_values(false)
                .help(format!("Sets logging level {:?}", *crate::LOG_LEVELS).as_str())
                .takes_value(true))
            .arg(Arg::with_name("log_style")
                .short("s")
                .long("log_style")
                .default_value("auto")
                .empty_values(false)
                .help(format!("Sets logging style {:?}", *crate::LOG_STYLES).as_str())
                .takes_value(true))
            .arg(Arg::with_name("ip")
                .short("i")
                .long("ip")
                .value_name("IP ADDR")
                .default_value("0.0.0.0")
                .empty_values(false)
                .help("Sets ip to bind services to.")
                .takes_value(true))
            .arg(Arg::with_name("pull_port")
                .short("p")
                .long("pull_port")
                .value_name("NUMBER")
                .default_value("7101")
                .empty_values(false)
                .validator(Args::is_good_port_number)
                .help("Sets port for PULL service.")
                .takes_value(true))
            .arg(Arg::with_name("pub_port")
                .short("b")
                .long("pub_port")
                .value_name("NUMBER")
                .default_value("7102")
                .empty_values(false)
                .validator(Args::is_good_port_number)
                .help("Sets port for PUB service.")
                .takes_value(true))
            .arg(Arg::with_name("rhwm")
                .short("w")
                .long("rhwm")
                .value_name("NUMBER")
                .default_value("1000")
                .empty_values(false)
                .validator(Args::hwm_is_sane)
                .help("Sets receive high water mark.")
                .takes_value(true))
            .arg(Arg::with_name("shwm")
                .short("s")
                .long("shwm")
                .value_name("NUMBER")
                .default_value("1000")
                .empty_values(false)
                .validator(Args::hwm_is_sane)
                .help("Sets send high water mark.")
                .takes_value(true))
            .get_matches();

        Args {
            rules_file: args.value_of("rules").unwrap().to_string(),
            log_level: args.value_of("log_level").unwrap().to_string(),
            log_style: args.value_of("log_style").unwrap().to_string(),
            ip: args.value_of("ip").unwrap().to_string(),
            pull_port: args.value_of("pull_port").unwrap().to_string().parse::<i32>().unwrap(),
            pub_port: args.value_of("pub_port").unwrap().to_string().parse::<i32>().unwrap(),
            rhwn: args.value_of("rhwm").unwrap().to_string().parse::<i32>().unwrap(),
            shwm: args.value_of("shwm").unwrap().to_string().parse::<i32>().unwrap(),
        }
    }
}