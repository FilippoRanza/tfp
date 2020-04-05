use clap::*;

use crate::net::{ClientMode, ServerMode};
use std::io::{Error, ErrorKind, Result};

pub enum Mode {
    Client(String, u16, ClientMode),
    Server(u16, ServerMode),
    Empty,
}

pub fn parse_args() -> Result<Mode> {
    let args = App::new("File Transfer")
        .subcommand(
            SubCommand::with_name("client")
                .arg(Arg::with_name("addr").required(true).takes_value(true))
                .arg(Arg::with_name("port").required(true).takes_value(true))
                .arg(Arg::with_name("files").takes_value(true).multiple(true))
                .arg(Arg::with_name("stop").short("-s").long("--stop"))
                .group(
                    ArgGroup::with_name("action")
                        .args(&["files", "stop"])
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("server")
                .arg(Arg::with_name("port").required(true).takes_value(true))
                .arg(
                    Arg::with_name("count")
                        .short("-c")
                        .long("--count")
                        .takes_value(true),
                )
                .arg(Arg::with_name("keep-on").short("-k").long("--keep"))
                .group(ArgGroup::with_name("mode").args(&["count", "keep-on"])),
        )
        .get_matches();

    let output = match args.subcommand() {
        ("client", Some(client_args)) => {
            let addr = client_args.value_of("addr").unwrap().to_owned();
            let port = parse_string(client_args.value_of("port").unwrap(), "port")?;
            let mode = if let Some(files) = client_args.values_of("files") {
                let tmp: Vec<String> = files.map(|s| s.to_owned()).collect();
                ClientMode::SendFile(tmp)
            } else {
                ClientMode::SendStop
            };
            Mode::Client(addr, port, mode)
        }
        ("server", Some(server_args)) => {
            let port = parse_string(server_args.value_of("port").unwrap(), "port")?;
            let mode = if let Some(count) = server_args.value_of("count") {
                let tmp = parse_string(count, "count")?;
                ServerMode::Count(tmp)
            } else if server_args.is_present("keep-on") {
                ServerMode::ForEver
            } else {
                ServerMode::Once
            };
            Mode::Server(port, mode)
        }
        ("", None) => Mode::Empty,
        _ => unreachable!(),
    };

    Ok(output)
}

fn parse_string(arg: &str, what: &str) -> Result<u16> {
    let result = arg.parse::<u16>();
    match result {
        Ok(port) => Ok(port),
        Err(err) => {
            let msg = format!("'{}' is an invalid {} number - {}", arg, what, err);
            let err = Error::new(ErrorKind::InvalidInput, msg);
            Err(err)
        }
    }
}
