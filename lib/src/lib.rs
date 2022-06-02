use binrw::{io::{Read, Seek}, prelude::*, until_eof, BinReaderExt, Error};
use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;

mod entries;
mod functions;
mod zip;

pub use entries::*;
pub use functions::*;
pub use zip::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Constants

    const VERBOSE: &str = include_str!("../../exercise.zip-process-verbose.txt");
    const SUMMARY: &str = include_str!("../../exercise.zip-process-summary.txt");

    // Process functions

    #[test]
    fn process_eof_test() {
        let bytes = hex::decode("00").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        assert_eq!(
            process(&mut reader, true).unwrap_err(),
            String::from("Unexpected end of file"),
        );
    }

    #[test]
    fn process_invalid_sig_test() {
        let bytes = hex::decode("00000001").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        assert_eq!(
            process(&mut reader, true).unwrap_err(),
            String::from("Invalid signature: `00000001`"),
        );
    }

    #[test]
    fn process_verbose_test() {
        let f = File::open("../exercise.zip").unwrap();
        let mut r = BufReader::new(f);
        assert_eq!(process(&mut r, true).unwrap(), VERBOSE);
    }

    #[test]
    fn process_summary_test() {
        let f = File::open("../exercise.zip").unwrap();
        let mut r = BufReader::new(f);
        assert_eq!(process(&mut r, false).unwrap(), SUMMARY);
    }

    #[test]
    fn process_file_nonexistent_test() {
        assert_eq!(
            process_file("nonexistent.zip", true).unwrap_err(),
            String::from("Path does not exist: `nonexistent.zip`"),
        );
    }

    #[test]
    fn process_file_not_file_test() {
        assert_eq!(
            process_file(".", true).unwrap_err(),
            String::from("Path is not a file: `.`"),
        );
    }

    #[test]
    fn process_file_test() {
        assert_eq!(process_file("../exercise.zip", true).unwrap(), VERBOSE);
    }

    #[test]
    fn process_file_summary_test() {
        assert_eq!(process_file("../exercise.zip", false).unwrap(), SUMMARY);
    }

    // Struct API

    #[test]
    fn zip_from_test() {
        let zip = Zip::from("../exercise.zip").unwrap();
        assert_eq!(zip.summary().unwrap(), SUMMARY);
        assert_eq!(zip.verbose().unwrap(), VERBOSE);
    }

    #[test]
    fn zip_from_nonexistent_test() {
        assert_eq!(
            Zip::from("nonexistent.zip").unwrap_err(),
            String::from("Path does not exist: `nonexistent.zip`"),
        );
    }

    #[test]
    fn zip_from_not_file_test() {
        assert_eq!(
            Zip::from(".").unwrap_err(),
            String::from("Path is not a file: `.`"),
        );
    }

    #[test]
    fn zip_process_test() {
        let f = File::open("../exercise.zip").unwrap();
        let mut r = BufReader::new(f);
        let zip = Zip::process(&mut r).unwrap();
        assert_eq!(zip.summary().unwrap(), SUMMARY);
        assert_eq!(zip.verbose().unwrap(), VERBOSE);
    }

    #[test]
    fn zip_process_eof_test() {
        let bytes = hex::decode("00").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        assert_eq!(
            Zip::process(&mut reader).unwrap_err(),
            String::from("Unexpected end of file"),
        );
    }

    #[test]
    fn zip_process_invalid_sig_test() {
        let bytes = hex::decode("00000001").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        assert_eq!(
            Zip::process(&mut reader).unwrap_err(),
            String::from("Invalid signature: `00000001`"),
        );
    }

}
