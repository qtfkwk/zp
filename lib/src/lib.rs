//! # About
//!
//! This library provides [`BinRead`](https://docs.rs/binrw/latest/binrw/trait.BinRead.html)
//! structs that represent a Zip file ([`Entries`]) and its components as a list of [`Entry`]
//! enums:
//!
//! * [`LocalFile`]: Local file header, file data, and data descriptor
//! * [`CentralDirectoryFileHeader`]: Central directory file header
//! * [`EndOfCentralDirectoryRecord`]: End of central directory record
//!
//! Each of these structs has a `verbose` method which is called by [`Zip::verbose`] to generate
//! a verbose representation of the data.
//!
//! The [`CentralDirectoryFileHeader`] also has a `summary` method which is called by
//! [`Zip::summary`] to generate a string with a tab-separated summary of the file with the file
//! name, whether it's a directory, uncompressed size, date/time, and file comment.
//!
//! # Struct API
//!
//! The primary API is provided via the [`Zip`] struct, which offers the [`Zip::from`] or
//! [`Zip::process`] methods to read zip file data from a file path or a [`BufReader`],
//! respectively.
//! Currently, the [`Zip`] struct offers two output methods, [`Zip::verbose`] and [`Zip::summary`],
//! formats, which show the zip file metadata in either verbose or summary format.
//!
//! ```
//! use zp_lib::Zip;
//!
//! let zip = Zip::from("../exercise.zip").unwrap();
//!
//! assert_eq!(
//!     zip.summary().unwrap(),
//!     "\
//! folder00/	true	0	2022-05-19T10:51:38	
//! folder00/folder00-00/	true	0	2022-05-19T10:51:18	A nested folder
//! folder00/folder00-00/test00-00-00.txt	false	4	2020-08-25T09:05:38	
//! folder00/folder00-00/test00-00-01.txt	false	125	2022-05-19T10:56:30	
//! folder00/folder00-00/test00-00-02.txt	false	4	2020-08-25T09:05:38	
//! folder00/test00-00.txt	false	95	2022-05-19T10:57:24	
//! folder00/test00-01.txt	false	0	2021-08-25T13:04:38	This file doesn't have any content
//! folder01/	true	0	2022-05-19T10:51:26	
//! folder01/exercise.zip	false	2272	2022-05-19T11:05:08	
//! folder01/test01-00.txt	false	127	2022-05-19T10:53:46	This is a comment
//! test00.txt	false	4	2020-08-25T09:05:38	A top level file
//! test01.txt	false	4	2020-08-25T09:05:38	
//! test02.txt	false	4	2020-08-25T09:05:38	
//! \
//!     ",
//! );
//! ```
//!
//! # Function API
//!
//! Also, the [`process_file`] and [`process`] functions enable reading zip file data from a file
//! path or a [`BufReader`] and specifying either verbose or summary output.
//!
//! ```
//! assert_eq!(
//!     zp_lib::process_file("../exercise.zip", false).unwrap(),
//!     "\
//! folder00/	true	0	2022-05-19T10:51:38	
//! folder00/folder00-00/	true	0	2022-05-19T10:51:18	A nested folder
//! folder00/folder00-00/test00-00-00.txt	false	4	2020-08-25T09:05:38	
//! folder00/folder00-00/test00-00-01.txt	false	125	2022-05-19T10:56:30	
//! folder00/folder00-00/test00-00-02.txt	false	4	2020-08-25T09:05:38	
//! folder00/test00-00.txt	false	95	2022-05-19T10:57:24	
//! folder00/test00-01.txt	false	0	2021-08-25T13:04:38	This file doesn't have any content
//! folder01/	true	0	2022-05-19T10:51:26	
//! folder01/exercise.zip	false	2272	2022-05-19T11:05:08	
//! folder01/test01-00.txt	false	127	2022-05-19T10:53:46	This is a comment
//! test00.txt	false	4	2020-08-25T09:05:38	A top level file
//! test01.txt	false	4	2020-08-25T09:05:38	
//! test02.txt	false	4	2020-08-25T09:05:38	
//! \
//!     ",
//! );
//! ```

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
