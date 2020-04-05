use std::io::Result;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;

use super::file_utils;
use super::status::{print_status_message, recv_status, send_status, Status};

pub enum ServerMode {
    Once,
    Count(u16),
    ForEver,
}

pub fn start_server(port: u16, mode: ServerMode) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr)?;
    match mode {
        ServerMode::Once => {
            run_once(&listener)?;
        }
        ServerMode::Count(c) => {
            for _ in 0..c {
                if !run_once(&listener)? {
                    break;
                }
            }
        }
        ServerMode::ForEver => loop {
            if !run_once(&listener)? {
                break;
            }
        },
    };
    Ok(())
}

fn run_once(listener: &TcpListener) -> Result<bool> {
    let (mut stream, _addr) = listener.accept()?;
    if let Status::Stop = recv_status(&mut stream)? {
        return Ok(false);
    }
    let file_count = parse_header(&mut stream)?;
    for _ in 0..file_count {
        let (stat, name) = handle_file_receive(&mut stream)?;
        print_status_message(stat, &name);
    }

    Ok(true)
}

fn handle_file_receive(stream: &mut TcpStream) -> Result<(Status, String)> {
    if let Some((name, size)) = parse_file_header(stream)? {
        if file_utils::exists(&name) {
            send_status(stream, Status::FileAlreadyExists)?;
            Ok((Status::FileAlreadyExists, name))
        } else {
            send_status(stream, Status::Ok)?;
            let stat = receive_file(stream, &name, size as usize)?;
            Ok((stat, name))
        }
    } else {
        send_status(stream, Status::NameError)?;
        Ok((Status::NameError, String::new()))
    }
}

fn parse_header(stream: &mut TcpStream) -> Result<i16> {
    let mut buff = [0; 2];
    stream.read_exact(&mut buff)?;
    Ok(i16::from_le_bytes(buff))
}

fn parse_file_header(stream: &mut TcpStream) -> Result<Option<(String, u64)>> {
    let mut name_len = [0; 1];
    let mut file_name = [0; 256];
    let mut file_size = [0; 8];

    stream.read_exact(&mut name_len)?;
    stream.read_exact(&mut file_name)?;
    stream.read_exact(&mut file_size)?;

    let name_len = name_len[0] as usize;
    let file_size = u64::from_le_bytes(file_size);
    match from_utf8(&file_name[..name_len]) {
        Ok(file_name) => Ok(Some((file_name.to_owned(), file_size))),
        Err(_) => Ok(None),
    }
}

fn receive_file(stream: &mut TcpStream, file_name: &str, mut file_size: usize) -> Result<Status> {
    let mut buff = [0; 2048];
    if let Status::FileOpenError = recv_status(stream)? {
        return Ok(Status::FileOpenError);
    }
    let mut file = file_utils::file_open(stream, file_name, file_utils::FileMode::Create)?;

    loop {
        if file_size > buff.len() {
            stream.read_exact(&mut buff)?;
            file.write_all(&buff)?;
            file_size -= buff.len();
        } else {
            stream.read_exact(&mut buff[..file_size])?;
            file.write_all(&buff[..file_size])?;
            break;
        }
    }

    Ok(Status::Ok)
}
