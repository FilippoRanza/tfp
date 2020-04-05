mod arg_parser;
mod net;

use std::io::{Error, ErrorKind, Result};

fn run_command(command: arg_parser::Mode) -> Result<()> {
    match command {
        arg_parser::Mode::Server(port, mode) => net::start_server(port, mode),
        arg_parser::Mode::Client(addr, port, mode) => net::start_client(&addr, port, mode),
        arg_parser::Mode::Empty => {
            let msg = "Missing 'server' or 'client' command";
            let err = Error::new(ErrorKind::InvalidInput, msg);
            Err(err)
        }
    }
}

fn run() -> Result<()> {
    let command = arg_parser::parse_args()?;
    run_command(command)
}

fn main() {
    let result = run();
    match result {
        Ok(()) => {}
        Err(err) => eprintln!("{}", err),
    }
}
