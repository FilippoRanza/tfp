use std::fs::metadata;
use std::path;

use std::fs::File;
use std::io::Result;
use std::net::TcpStream;

use super::status::{send_status, Status};

pub enum FileMode {
    Open,
    Create,
}

pub fn file_open(stream: &mut TcpStream, file_name: &str, mode: FileMode) -> Result<File> {
    let stat = match mode {
        FileMode::Open => File::open(file_name),
        FileMode::Create => File::create(file_name),
    };
    match stat {
        Ok(file) => {
            send_status(stream, Status::Ok)?;
            Ok(file)
        }
        Err(err) => {
            send_status(stream, Status::FileOpenError)?;
            return Err(err);
        }
    }
}

pub fn convert_name(file_name: &str) -> ([u8; 256], u8) {
    let path = path::Path::new(file_name);
    let file_name = if path.is_absolute() {
        path.file_name().unwrap().to_str().unwrap()
    } else {
        file_name
    };
    let len = file_name.len() as u8;
    let mut output = [0; 256];
    for (i, b) in file_name.as_bytes().iter().enumerate() {
        output[i] = *b;
    }
    (output, len)
}

pub fn file_size(name: &str) -> Result<u64> {
    let meta = metadata(name)?;
    let output = meta.len();
    Ok(output)
}

pub fn exists(name: &str) -> bool {
    let path = path::Path::new(name);
    path.exists()
}

pub fn clean_file_list(files: &[String]) -> Vec<&String> {
    files.iter().filter(|n| check_file(n)).collect()
}

fn check_file(name: &str) -> bool {
    let path = path::Path::new(name);
    if path.exists() {
        if path.is_file() {
            true
        } else {
            println!("{} is not a regular file", name);
            false
        }
    } else {
        println!("{} does not exists, it will be ignored", name);
        false
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_convert_name() {
        let path = if cfg!(windows) {
            r"C:\system\user\file"
        } else {
            r"/system/user/file"
        };

        let (bytes, len) = convert_name(path);
        assert_eq!(len, 4);
        let mut ans: [u8; 256] = [0; 256];
        ans[0] = 102;
        ans[1] = 105;
        ans[2] = 108;
        ans[3] = 101;
        for (a, b) in ans.iter().zip(bytes.iter()) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_check_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_name = temp_dir.path().join("name");
        let _file = File::create(&file_name).unwrap();
        assert!(check_file(file_name.to_str().unwrap()));
        assert!(!check_file("non-existing.not"));
        assert!(!check_file(temp_dir.path().to_str().unwrap()));
    }
}
