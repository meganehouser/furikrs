#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate clap;
extern crate failure;
extern crate github_rs;
extern crate pit;
extern crate serde_json;

mod activity;
mod client;
mod dateutil;
mod json;
mod markdown;

use std::io::{self, Write};
use chrono::Local;
use clap::{Arg, app_from_crate, crate_authors, crate_name, crate_description, crate_version};
use failure::Error;
use pit::Pit;
use crate::client::ActivityClient;
use crate::dateutil::naive_str_to_utc;
use crate::markdown::Markdown;

fn main() -> Result<(), Error> {
    env_logger::init();

    let today = Local::today().format("%Y-%m-%d").to_string();

    let matches = app_from_crate!()
        .arg(Arg::with_name("private")
            .short("p")
            .long("private")
            .help("enable get data from private repositories. (default: disable)"))
        .arg(Arg::with_name("from date")
            .short("f")
            .long("from-date")
            .default_value(&today)
            .help("set start of date formatted 'YYYY-MM-DD. (default: current day)"))
        .arg(Arg::with_name("to date")
            .short("t")
            .long("to-date")
            .default_value(&today)
            .help("set end of date formatted 'YYYY-MM-DD. (default: current day)"))
        .get_matches();

    let include_private = matches.is_present("private");

    let from_datetime = naive_str_to_utc(
        matches.value_of("from date").unwrap(),
        "00:00:00.000000")
        .unwrap();
    let to_datetime = naive_str_to_utc(
        matches.value_of("to date").unwrap(),
        "23:59:59.999999")
        .unwrap();

    let pit = Pit::new();
    let config = pit.get("github.com").expect("not provide config value of github.com");
    let access_token = config.get("access_token").expect("not provide access_token from config");
    let user_name = config.get("user_name").expect("not provide access_token from config");

    let client = ActivityClient::new(user_name, access_token);
    let activities = client.collect(&from_datetime, &to_datetime, include_private)?;

    let mut out = io::stdout();
    activities.write_markdown(&mut out)?;
    out.flush()?;

    Ok(())
}
