use std::io::Result;
use std::io::{Read, Write};
use std::net::TcpStream;

pub enum Status {
    Ok,
    Abort,
    FileAlreadyExists,
    NameError,
    FileOpenError,
    Stop,
}

pub fn from_u8(n: u8) -> Status {
    match n {
        0 => Status::Ok,
        1 => Status::Abort,
        2 => Status::FileAlreadyExists,
        3 => Status::NameError,
        4 => Status::FileOpenError,
        5 => Status::Stop,
        _ => panic!(),
    }
}

pub fn to_u8(stat: Status) -> u8 {
    match stat {
        Status::Ok => 0,
        Status::Abort => 1,
        Status::FileAlreadyExists => 2,
        Status::NameError => 3,
        Status::FileOpenError => 4,
        Status::Stop => 5,
    }
}

pub fn recv_status(stream: &mut TcpStream) -> Result<Status> {
    let mut buff = [0; 1];
    stream.read_exact(&mut buff)?;

    let code = buff[0];
    Ok(from_u8(code))
}

pub fn send_status(stream: &mut TcpStream, stat: Status) -> Result<()> {
    let val = to_u8(stat);
    stream.write_all(&[val])?;
    Ok(())
}

pub fn print_status_message(stat: Status, file_name: &str) {
    match stat {
        Status::Ok => {}
        Status::Abort => println!("peer aborted this operation, exit now"),
        Status::FileAlreadyExists => {
            println!("A file named {} already exists on the server", file_name)
        }
        Status::FileOpenError => println!("the peer cannot open the file {}", file_name),
        Status::NameError => println!("the peer cannot convert received bytes into a String"),
        Status::Stop => println!("client is stopping server operation, no file will be transfered"),
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_convertion() {
        let statuses = 4;
        for i in 0..statuses {
            assert_eq!(to_u8(from_u8(i)), i);
        }
    }
}
