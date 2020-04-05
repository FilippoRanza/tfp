pub mod client;
pub mod server;

mod file_utils;
mod status;

pub use client::{start_client, ClientMode};
pub use server::{start_server, ServerMode};

#[cfg(test)]
mod test {

    use lazy_static::lazy_static;
    use rand::prelude::*;
    use std::env::set_current_dir;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    use super::*;

    lazy_static! {
        static ref DEST_DIR: TempDir = TempDir::new().unwrap();
    }

    #[test]
    fn test_simple_file_transfer() {
        let src_dir = TempDir::new().unwrap();

        let src_file = src_dir.path().join("file.txt").to_str().unwrap().to_owned();

        let mut file = File::create(&src_file).unwrap();
        file.write_all(&[1, 2, 3, 4]).unwrap();

        let tmp_copy = DEST_DIR.path().to_owned();
        let thread = thread::spawn(move || {
            set_current_dir(tmp_copy).unwrap();
            start_server(1111, ServerMode::Once).unwrap();
        });

        start_client("localhost", 1111, ClientMode::SendFile(vec![src_file])).unwrap();
        thread.join().unwrap();

        let dest_file = DEST_DIR.path().join("file.txt");
        assert!(dest_file.exists());
        assert_eq!(
            file_utils::file_size(dest_file.to_str().unwrap()).unwrap(),
            4
        );
        let mut file = File::open(&dest_file).unwrap();
        let mut data = [0; 4];
        file.read_exact(&mut data).unwrap();
        for (i, v) in data.iter().enumerate() {
            assert_eq!(i + 1, *v as usize);
        }
    }

    #[test]
    fn test_multiple_file_transfer() {
        let files = [
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        ];
        let src_dir = make_dir_and_files(&files[2..], 5);

        let tmp_copy = DEST_DIR.path().to_owned();
        let thread = thread::spawn(move || {
            set_current_dir(tmp_copy).unwrap();
            start_server(2222, ServerMode::Once).unwrap();
        });

        let files_path: Vec<String> = files
            .iter()
            .map(|f| src_dir.path().join(f).to_str().unwrap().to_owned())
            .collect();
        start_client("localhost", 2222, ClientMode::SendFile(files_path)).unwrap();
        thread.join().unwrap();

        let dest_path = DEST_DIR.path();
        for file in &files[..2] {
            let tmp = dest_path.join(file);
            assert!(!tmp.exists());
        }

        for file in &files[2..] {
            let tmp = dest_path.join(file);
            assert!(tmp.exists());
        }
    }

    #[test]
    fn test_multiple_connections() {
        let files = [
            "A".to_owned(),
            "B".to_owned(),
            "C".to_owned(),
            "D".to_owned(),
        ];
        let src_dir = make_dir_and_files(&files, 5);

        let tmp_copy = DEST_DIR.path().to_owned();
        let thread = thread::spawn(move || {
            set_current_dir(tmp_copy).unwrap();
            start_server(3333, ServerMode::Count(4)).unwrap();
        });

        for file in files.iter().map(|f| src_dir.path().join(f)) {
            thread::sleep(Duration::from_millis(10));
            start_client(
                "localhost",
                3333,
                ClientMode::SendFile(vec![file.to_str().unwrap().to_owned()]),
            )
            .unwrap();
        }

        thread.join().unwrap();

        let dest_path = DEST_DIR.path();

        for file in &files {
            let tmp = dest_path.join(file);
            assert!(tmp.exists());
        }
    }

    #[test]
    #[should_panic(expected = "Connection refused")]
    fn test_multiple_connections_with_failure() {
        let files = [
            "Aa".to_owned(),
            "Ba".to_owned(),
            "Ca".to_owned(),
            "Da".to_owned(),
        ];
        let src_dir = make_dir_and_files(&files, 5);

        let tmp_copy = DEST_DIR.path().to_owned();
        let thread = thread::spawn(move || {
            set_current_dir(tmp_copy).unwrap();
            start_server(4444, ServerMode::Count(3)).unwrap();
        });

        for file in files.iter().map(|f| src_dir.path().join(f)) {
            thread::sleep(Duration::from_millis(10));
            start_client(
                "localhost",
                4444,
                ClientMode::SendFile(vec![file.to_str().unwrap().to_owned()]),
            )
            .unwrap();
        }

        thread.join().unwrap();

        let dest_path = DEST_DIR.path();

        for file in &files {
            let tmp = dest_path.join(file);
            assert!(tmp.exists());
        }
    }

    #[test]
    #[should_panic(expected = "Connection refused")]
    fn test_stop_multiple_connections() {
        let files = [
            "Ab".to_owned(),
            "Bb".to_owned(),
            "Cb".to_owned(),
            "Db".to_owned(),
        ];
        let src_dir = make_dir_and_files(&files, 5);

        let tmp_copy = DEST_DIR.path().to_owned();
        let thread = thread::spawn(move || {
            set_current_dir(tmp_copy).unwrap();
            start_server(5555, ServerMode::ForEver).unwrap();
        });

        start_client("localhost", 5555, ClientMode::SendStop).unwrap();

        for file in files.iter().map(|f| src_dir.path().join(f)) {
            thread::sleep(Duration::from_millis(10));
            start_client(
                "localhost",
                5555,
                ClientMode::SendFile(vec![file.to_str().unwrap().to_owned()]),
            )
            .unwrap();
        }
        thread.join().unwrap();
    }

    fn make_dir_and_files(files: &[String], rand_data: usize) -> TempDir {
        let mut rng = thread_rng();
        let dir = TempDir::new().unwrap();
        let dir_path = dir.path();
        for file in files {
            let tmp = dir_path.join(file);
            let mut file = File::create(tmp).unwrap();
            if rand_data > 0 {
                let vec = make_rand_vect(rand_data, &mut rng);
                file.write_all(&vec).unwrap();
            }
        }

        dir
    }

    fn make_rand_vect(size: usize, rng: &mut rand::prelude::ThreadRng) -> Vec<u8> {
        (0..size).map(|_| rng.gen()).collect()
    }
}
