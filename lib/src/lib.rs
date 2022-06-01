use binrw::{prelude::*, until_eof, BinReaderExt, Error};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::PathBuf;

// Function API

/// Process a zip file at path
pub fn process_file(path: &str, verbose: bool) -> Result<String, String> {
    Zip::from(path)?.output(verbose)
}

/// Process the raw bytes of a zip file
///
/// If `verbose` is true: returns a complete analysis of the zip file contents.
///
/// If `verbose` is false: return a summary of the the zip file contents (file name, whether item
/// is a folder, uncompressed size, modified date/time, and comment).
pub fn process<R>(r: &mut BufReader<R>, verbose: bool) -> Result<String, String>
where
    R: Read + Seek,
{
    Zip::process(r)?.output(verbose)
}

// Conversion functions

/// Convert a u16 into a `((hours, minutes, seconds), u16)`
///
/// stored in standard MS-DOS format:
///
/// * Bits 00-04: seconds divided by 2
/// * Bits 05-10: minute
/// * Bits 11-15: hour
fn mod_time(n: u16) -> ((u8, u8, u8), u16) {
    let h = (n >> 11) as u8;
    let m = ((n >> 5) & (0b0000000000111111 as u16)) as u8;
    let s = 2 * (n & (0b0000000000011111 as u16)) as u8;
    ((h, m, s), n)
}

/// Convert a u16 into `((year, month, day), u16)`
///
/// stored in standard MS-DOS format:
///
/// * Bits 00-04: day
/// * Bits 05-08: month
/// * Bits 09-15: years from 1980
fn mod_date(n: u16) -> ((u16, u8, u8), u16) {
    let y = (n >> 9) + 1980u16;
    let m = ((n >> 5) & (0b0000000000001111 as u16)) as u8;
    let d = (n & (0b0000000000011111 as u16)) as u8;
    ((y, m, d), n)
}

/// Convert the debug string of a `binrw::Error::BadMagic.found` into a nice hex string
fn magic_hex(magic: &str) -> String {
    magic
        .replace("[", "")
        .replace("]", "")
        .split(", ")
        .map(|x| format!("{:02x}", x.parse::<u8>().unwrap()))
        .collect::<Vec<String>>()
        .join("")
}

// Struct API

#[derive(Debug)]
pub struct Zip {
    path: Option<PathBuf>,
    entries: Entries,
}

impl Zip {
    /// Process a zip file at path
    pub fn from<P>(path: P) -> Result<Self, String>
    where
        P: Into<PathBuf>,
    {
        let path: PathBuf = path.into();
        if !path.exists() {
            return Err(format!("Path does not exist: `{}`", path.display()));
        } else if !path.is_file() {
            return Err(format!("Path is not a file: `{}`", path.display()));
        }
        match File::open(&path) {
            Ok(f) => {
                let mut r = BufReader::new(f);
                let mut zip = Zip::process(&mut r)?;
                zip.path = Some(path);
                Ok(zip)
            }
            Err(e) => Err(format!("{e}: `{}`", path.display())),
        }
    }

    /// Process the raw bytes of a zip file
    pub fn process<R>(r: &mut BufReader<R>) -> Result<Self, String>
    where
        R: Read + Seek,
    {
        match r.read_le::<Entries>() {
            Ok(entries) => {
                if entries.list.is_empty() {
                    Err(String::from("Unexpected end of file"))
                } else {
                    Ok(Self {
                        path: None,
                        entries,
                    })
                }
            }
            Err(e) => {
                let e = e.root_cause(); // not the backtrace error

                // Check if the outer error is an `EnumErrors`
                if let Error::EnumErrors {
                    pos: _,
                    variant_errors,
                } = &e
                {
                    // Count the bad magic errors and save the magic value seen
                    let mut count_bad_magic = 0;
                    let mut magic = None;
                    for (_, i) in variant_errors {
                        let i = i.root_cause();
                        if let Error::BadMagic { pos: _, found } = &i {
                            magic = Some(magic_hex(&format!("{:?}", found)));
                            count_bad_magic += 1;
                        }
                    }

                    // If got a magic value and all the variant errors were bad magic errors,
                    // return an invalid signature error.
                    if magic.is_some() && count_bad_magic == variant_errors.len() {
                        return Err(format!("Invalid signature: `{}`", magic.as_ref().unwrap()));
                    }
                }

                // Return the error
                Err(e.to_string())
            }
        }
    }

    /// Helper to call `verbose()` or `summary()` based on the value of `verbose`
    pub fn output(&self, verbose: bool) -> Result<String, String> {
        if verbose {
            self.verbose()
        } else {
            self.summary()
        }
    }

    /// Generate a complete analysis of the zip file contents
    pub fn verbose(&self) -> Result<String, String> {
        let mut s = vec![];
        for entry in &self.entries.list {
            s.push(String::from("---\n"));
            match entry {
                Entry::LocalFile(i) => s.push(i.verbose()),
                Entry::CentralDirectoryFileHeader(i) => s.push(i.verbose()),
                Entry::EndOfCentralDirectoryRecord(i) => s.push(i.verbose()),
            }
        }
        s.push(String::from("---\nEOF\n---\n"));
        Ok(s.join(""))
    }

    /// Generate a summary of the the zip file contents
    /// (file name, whether item is a folder, uncompressed size, modified date/time, and comment)
    pub fn summary(&self) -> Result<String, String> {
        let mut s = vec![];
        for entry in &self.entries.list {
            if let Entry::CentralDirectoryFileHeader(i) = entry {
                s.push(i.summary());
            }
        }
        Ok(s.join(""))
    }
}

// Zip file representation via binrw

#[derive(BinRead, Debug)]
struct Entries {
    #[br(parse_with = until_eof)]
    list: Vec<Entry>,
}

#[derive(BinRead, Debug)]
enum Entry {
    LocalFile(LocalFile),
    CentralDirectoryFileHeader(CentralDirectoryFileHeader),
    EndOfCentralDirectoryRecord(EndOfCentralDirectoryRecord),
}

#[derive(BinRead, Debug)]
#[br(magic = b"\x50\x4b\x03\x04")]
pub struct LocalFile {
    version: u16,
    flags: u16,
    compression: u16,
    mod_time: u16,
    mod_date: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    extra_field_length: u16,

    #[br(count = file_name_length)]
    file_name: Vec<u8>,

    #[br(count = extra_field_length)]
    extra_field: Vec<u8>,

    #[br(count = compressed_size)]
    file_data: Vec<u8>,

    #[br(if(flags & (1 << 3) != 0))]
    data_descriptor: Option<DataDescriptor>,
}

#[derive(BinRead, Debug)]
pub struct DataDescriptor {
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
}

#[derive(BinRead, Debug)]
#[br(magic = b"\x50\x4b\x01\x02")]
pub struct CentralDirectoryFileHeader {
    version: u16,
    version_needed: u16,
    flags: u16,
    compression: u16,
    mod_time: u16,
    mod_date: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    extra_field_length: u16,
    file_comment_length: u16,
    disk_number_start: u16,
    internal_file_attributes: u16,
    external_file_attributes: u32,
    lfh_offset: u32,

    #[br(count = file_name_length)]
    file_name: Vec<u8>,

    #[br(count = extra_field_length)]
    extra_field: Vec<u8>,

    #[br(count = file_comment_length)]
    file_comment: Vec<u8>,
}

#[derive(BinRead, Debug)]
#[br(magic = b"\x50\x4b\x05\x06")]
pub struct EndOfCentralDirectoryRecord {
    disk_number: u16,
    disk_number_w_cd: u16,
    disk_entries: u16,
    total_entries: u16,
    cd_size: u32,
    cd_offset: u32,
    comment_length: u16,

    #[br(count = comment_length)]
    zip_file_comment: Vec<u8>,
}

// Output methods

impl LocalFile {
    fn verbose(&self) -> String {
        format!(
            "\
sig = 0x504b0304 (Local file header)
version = 0x{:04x} ({})
flags = 0x{:04x} ({})
compression = 0x{:04x} ({})
mod_time = 0x{:04x} ({:?})
mod_date = 0x{:04x} ({:?})
crc32 = 0x{:08x} ({})
compressed_size = 0x{:08x} ({})
uncompressed_size = 0x{:08x} ({})
file_name_length = 0x{:04x} ({})
extra_field_length = 0x{:04x} ({})
file_name = {:?} ({:?})
extra_field = {:?}
file_data = {:?}
data_descriptor = {}
\
            ",
            self.version,
            self.version,
            self.flags,
            self.flags,
            self.compression,
            self.compression,
            self.mod_time,
            mod_time(self.mod_time).0,
            self.mod_date,
            mod_date(self.mod_date).0,
            self.crc32,
            self.crc32,
            self.compressed_size,
            self.compressed_size,
            self.uncompressed_size,
            self.uncompressed_size,
            self.file_name_length,
            self.file_name_length,
            self.extra_field_length,
            self.extra_field_length,
            hex::encode(&self.file_name),
            std::str::from_utf8(&self.file_name).unwrap(),
            hex::encode(&self.extra_field),
            hex::encode(&self.file_data),
            match &self.data_descriptor {
                Some(d) => d.verbose(),
                None => String::from("None"),
            },
        )
    }
}

impl DataDescriptor {
    fn verbose(&self) -> String {
        format!(
            "\
{{
    crc32 = 0x{:08x} ({})
    compressed_size = 0x{:08x} ({})
    uncompressed_size = 0x{:08x} ({})
}}
\
            ",
            self.crc32,
            self.crc32,
            self.compressed_size,
            self.compressed_size,
            self.uncompressed_size,
            self.uncompressed_size,
        )
    }
}

impl CentralDirectoryFileHeader {
    fn verbose(&self) -> String {
        format!(
            "\
sig = 0x504b0102 (Central directory file header)
version = 0x{:04x} ({})
version_needed = 0x{:04x} ({})
flags = 0x{:04x} ({})
compression = 0x{:04x} ({})
mod_time = 0x{:04x} ({:?})
mod_date = 0x{:04x} ({:?})
crc32 = 0x{:08x} ({})
compressed_size = 0x{:08x} ({})
uncompressed_size = 0x{:08x} ({})
file_name_length = 0x{:04x} ({})
extra_field_length = 0x{:04x} ({})
file_comment_length = 0x{:04x} ({})
disk_number_start = 0x{:04x} ({})
internal_file_attributes = 0x{:04x} ({})
external_file_attributes = 0x{:08x} ({})
lfh_offset = 0x{:08x} ({})
file_name = {:?} ({:?})
extra_field = {:?}
file_comment = {:?} ({:?})
\
            ",
            self.version,
            self.version,
            self.version_needed,
            self.version_needed,
            self.flags,
            self.flags,
            self.compression,
            self.compression,
            self.mod_time,
            mod_time(self.mod_time).0,
            self.mod_date,
            mod_date(self.mod_date).0,
            self.crc32,
            self.crc32,
            self.compressed_size,
            self.compressed_size,
            self.uncompressed_size,
            self.uncompressed_size,
            self.file_name_length,
            self.file_name_length,
            self.extra_field_length,
            self.extra_field_length,
            self.file_comment_length,
            self.file_comment_length,
            self.disk_number_start,
            self.disk_number_start,
            self.internal_file_attributes,
            self.internal_file_attributes,
            self.external_file_attributes,
            self.external_file_attributes,
            self.lfh_offset,
            self.lfh_offset,
            hex::encode(&self.file_name),
            std::str::from_utf8(&self.file_name).unwrap(),
            hex::encode(&self.extra_field),
            hex::encode(&self.file_comment),
            std::str::from_utf8(&self.file_comment).unwrap(),
        )
    }

    fn summary(&self) -> String {
        let t = mod_time(self.mod_time).0;
        let d = mod_date(self.mod_date).0;
        format!(
            "{}\t{}\t{}\t{:04}-{:02}-{:02}T{:02}:{:02}:{:02}\t{}\n",
            std::str::from_utf8(&self.file_name).unwrap(),
            self.file_name.ends_with(b"/"),
            self.uncompressed_size,
            d.0,
            d.1,
            d.2,
            t.0,
            t.1,
            t.2,
            std::str::from_utf8(&self.file_comment).unwrap(),
        )
    }
}

impl EndOfCentralDirectoryRecord {
    fn verbose(&self) -> String {
        format!(
            "\
sig = 0x504b0506 (End of central directory record)
disk_number = 0x{:04x} ({})
disk_number_w_cd = 0x{:04x} ({})
disk_entries = 0x{:04x} ({})
total_entries = 0x{:04x} ({})
cd_size = 0x{:08x} ({})
cd_offset = 0x{:08x} ({})
comment_length = 0x{:04x} ({})
zip_file_comment = {:?} ({:?})
\
            ",
            self.disk_number,
            self.disk_number,
            self.disk_number_w_cd,
            self.disk_number_w_cd,
            self.disk_entries,
            self.disk_entries,
            self.total_entries,
            self.total_entries,
            self.cd_size,
            self.cd_size,
            self.cd_offset,
            self.cd_offset,
            self.comment_length,
            self.comment_length,
            hex::encode(&self.zip_file_comment),
            std::str::from_utf8(&self.zip_file_comment).unwrap(),
        )
    }
}

// Tests

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
        let f = File::open("../../exercise.zip").unwrap();
        let mut r = BufReader::new(f);
        assert_eq!(process(&mut r, true).unwrap(), VERBOSE);
    }

    #[test]
    fn process_summary_test() {
        let f = File::open("../../exercise.zip").unwrap();
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
        assert_eq!(process_file("../../exercise.zip", true).unwrap(), VERBOSE);
    }

    #[test]
    fn process_file_summary_test() {
        assert_eq!(process_file("../../exercise.zip", false).unwrap(), SUMMARY);
    }

    // Struct API

    #[test]
    fn zip_from_test() {
        let zip = Zip::from("../../exercise.zip").unwrap();
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
        let f = File::open("../../exercise.zip").unwrap();
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
