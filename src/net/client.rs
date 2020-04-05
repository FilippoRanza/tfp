use std::io::Result;
use std::io::{Read, Write};
use std::net::TcpStream;

use super::file_utils;
use super::status::{print_status_message, recv_status, send_status, Status};

pub enum ClientMode {
    SendFile(Vec<String>),
    SendStop,
}

pub fn start_client(addr: &str, port: u16, mode: ClientMode) -> Result<()> {
    let addr = format!("{}:{}", addr, port);
    let mut connection = TcpStream::connect(&addr)?;
    match mode {
        ClientMode::SendFile(files) => send_files(&mut connection, &files)?,
        ClientMode::SendStop => send_status(&mut connection, Status::Stop)?,
    }
    Ok(())
}

fn send_files(stream: &mut TcpStream, files: &[String]) -> Result<()> {
    send_status(stream, Status::Ok)?;
    let files = file_utils::clean_file_list(files);
    send_header(stream, files.len() as i16)?;
    for f in files {
        let stat = handle_file_send(stream, f)?;
        print_status_message(stat, f);
    }
    Ok(())
}

fn handle_file_send(stream: &mut TcpStream, file_name: &str) -> Result<Status> {
    let size = file_utils::file_size(file_name)?;
    file_header(stream, file_name, size)?;

    let stat = recv_status(stream)?;
    if let Status::Ok = stat {
        let stat = send_file(stream, file_name)?;
        Ok(stat)
    } else {
        Ok(stat)
    }
}

fn send_header(stream: &mut TcpStream, file_count: i16) -> Result<()> {
    let buff = file_count.to_le_bytes();
    stream.write_all(&buff)?;
    Ok(())
}

fn file_header(stream: &mut TcpStream, file_name: &str, file_size: u64) -> Result<()> {
    let (file_name, name_len) = file_utils::convert_name(file_name);
    let file_size = file_size.to_le_bytes();

    stream.write_all(&[name_len])?;
    stream.write_all(&file_name)?;
    stream.write_all(&file_size)?;

    Ok(())
}

fn send_file(stream: &mut TcpStream, file_name: &str) -> Result<Status> {
    let mut file = file_utils::file_open(stream, file_name, file_utils::FileMode::Open)?;

    if let Status::FileOpenError = recv_status(stream)? {
        return Ok(Status::FileOpenError);
    }

    let mut buff = [0; 2048];
    loop {
        let amount = file.read(&mut buff)?;
        if amount == 0 {
            break;
        }
        stream.write_all(&buff[..amount])?;
    }

    Ok(Status::Ok)
}
