extern crate clap;
extern crate serialport;
extern crate reqwest;


use std::io::BufReader;
use std::io::BufRead;
use std::str;
use std::time::Duration;
use std::collections::HashMap;

use clap::{App, AppSettings, Arg};
use serialport::prelude::*;

struct Config {
    username: String,
    feed: String,
    token: String
}

fn main() {
    let matches = App::new("Serialport Example - Receive Data")
        .about("Reads data from a serial port and echoes it to stdout")
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("port")
                .help("The device path to a serial port")
                .use_delimiter(false)
                .required(true),
        )
        .arg(
            Arg::with_name("baud")
                .help("The baud rate to connect at")
                .use_delimiter(false)
                .required(true),
        )
        .arg(
            Arg::with_name("token")
                .help("The Adafruit token")
                .use_delimiter(false)
                .required(true),
        )
        .get_matches();

    let port_name = matches.value_of("port").unwrap();
    let baud_rate = matches.value_of("baud").unwrap();
    let token = matches.value_of("token").unwrap();

    let config = Config {
        username: "andygrove".to_owned(),
        feed: "office".to_owned(),
        token: token.to_owned()
    };

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    if let Ok(rate) = baud_rate.parse::<u32>() {
        settings.baud_rate = rate.into();
    } else {
        eprintln!("Error: Invalid baud rate '{}' specified", baud_rate);
        ::std::process::exit(1);
    }

    match serialport::open_with_settings(&port_name, &settings) {
        Ok(port) => {
            let mut reader = BufReader::new(port);
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(_) => process(&config,&line),
                    Err(_) => {}
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}

fn process(config: &Config, s: &str) {
    println!("Received: {}", s);

    if s.starts_with("H2S_ppb=") {
        let h2s = &s[8..].to_string();

        println!("Posting value {}", h2s);

        let mut map = HashMap::new();
        map.insert("value", h2s);

        let client = reqwest::blocking::Client::new();

        let url = format!("https://io.adafruit.com/api/v2/{}/feeds/{}/data", &config.username, &config.feed);

        let res = client.post(&url)
            .header("X-AIO-Key", &config.token)
            .json(&map)
            .send()
            .unwrap();

        println!("{:?}", res);
    }

    //io::stdout().write_all(&serial_buf[..t]).unwrap(),
}
