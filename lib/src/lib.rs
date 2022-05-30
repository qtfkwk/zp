use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

// Field enum

enum Field {
    U16LE,
    U32LE,
    U32BE,
}

impl Field {
    /// Length of the field in bytes
    fn len(&self) -> usize {
        match self {
            Self::U16LE => 2,
            Self::U32LE => 4,
            Self::U32BE => 4,
        }
    }
}

// Conversion functions

/// Convert 2 u8's (little endian) into `((hours, minutes, seconds), u16)`
///
/// stored in standard MS-DOS format:
///
/// * Bits 00-04: seconds divided by 2
/// * Bits 05-10: minute
/// * Bits 11-15: hour
fn mod_time(a: &[u8]) -> ((u8, u8, u8), u16) {
    let n = a
        .iter()
        .take(2)
        .enumerate()
        .map(|(i, x)| (*x as u16) << (8 * i))
        .sum();
    let h = (n >> 11) as u8;
    let m = ((n >> 5) & (0b0000000000111111 as u16)) as u8;
    let s = 2 * (n & (0b0000000000011111 as u16)) as u8;
    ((h, m, s), n)
}

/// Convert 2 u8's (little endian) into `((year, month, day), u16)`
///
/// stored in standard MS-DOS format:
///
/// * Bits 00-04: day
/// * Bits 05-08: month
/// * Bits 09-15: years from 1980
fn mod_date(a: &[u8]) -> ((u16, u8, u8), u16) {
    let n = a
        .iter()
        .take(2)
        .enumerate()
        .map(|(i, x)| (*x as u16) << (8 * i))
        .sum();
    let y = (n >> 9) + 1980u16;
    let m = ((n >> 5) & (0b0000000000001111 as u16)) as u8;
    let d = (n & (0b0000000000011111 as u16)) as u8;
    ((y, m, d), n)
}

/// Convert 2 u8's (little endian) into a u16
fn u16le(a: &[u8]) -> u16 {
    a.iter()
        .take(2)
        .enumerate()
        .map(|(i, x)| (*x as u16) << (8 * i))
        .sum()
}

/// Convert 4 u8's (little endian) into a u32
fn u32le(a: &[u8]) -> u32 {
    a.iter()
        .take(4)
        .enumerate()
        .map(|(i, x)| (*x as u32) << (8 * i))
        .sum()
}

/// Convert 4 u8's (big endian) into a u32
fn u32be(a: &[u8]) -> u32 {
    a.iter()
        .take(4)
        .rev()
        .enumerate()
        .map(|(i, x)| (*x as u32) << (8 * i))
        .sum()
}

// Read functions

/// Read `n` bytes from a `BufReader`
fn read_n<R>(n: usize, r: &mut BufReader<R>) -> Result<Vec<u8>, String>
where
    R: Read,
{
    let mut buf = vec![0; n];
    match r.read_exact(&mut buf) {
        Ok(()) => Ok(buf),
        Err(e) => Err(format!("{e}")),
    }
}

/// Read correct number of bytes for a `Field` from a `BufReader`
fn read<R>(field: Field, r: &mut BufReader<R>) -> Result<Vec<u8>, String>
where
    R: Read,
{
    read_n(field.len(), r)
}

/// Read a file modification time from a `BufReader`
fn read_mod_time<R>(r: &mut BufReader<R>) -> Result<((u8, u8, u8), u16), String>
where
    R: Read,
{
    Ok(mod_time(&read(Field::U16LE, r)?))
}

/// Read a file modification date from a `BufReader`
fn read_mod_date<R>(r: &mut BufReader<R>) -> Result<((u16, u8, u8), u16), String>
where
    R: Read,
{
    Ok(mod_date(&read(Field::U16LE, r)?))
}

/// Read a u16 (little endian) from a `BufReader`
fn read_u16le<R>(r: &mut BufReader<R>) -> Result<u16, String>
where
    R: Read,
{
    Ok(u16le(&read(Field::U16LE, r)?))
}

/// Read a u32 (little endian) from a `BufReader`
fn read_u32le<R>(r: &mut BufReader<R>) -> Result<u32, String>
where
    R: Read,
{
    Ok(u32le(&read(Field::U32LE, r)?))
}

/// Read a u32 (big endian) from a `BufReader`
fn read_u32be<R>(r: &mut BufReader<R>) -> Result<u32, String>
where
    R: Read,
{
    Ok(u32be(&read(Field::U32BE, r)?))
}

// Process functions

/// Process the raw bytes of a zip file
///
/// If `verbose` is true: returns a complete analysis of the zip file contents.
///
/// If `verbose` is false: return a summary of the the zip file contents (file name, whether item
/// is a folder, uncompressed size, modified date/time, and comment).
fn process<R>(r: &mut BufReader<R>, verbose: bool) -> Result<String, String>
where
    R: Read,
{
    let mut s = vec![];

    loop {
        if verbose {
            s.push(String::from("---\n"));
        }

        // Signature
        match read_u32be(r) {
            Ok(sig) => {
                // Local file header
                if sig == 0x504b0304 {
                    if verbose {
                        s.push(format!("sig = 0x{:08x} (Local file header)\n", sig));
                    }

                    // Version
                    let version = read_u16le(r)?;
                    if verbose {
                        s.push(format!("version = 0x{:04x} ({})\n", version, version));
                    }

                    // Flags
                    let flags = read_u16le(r)?;
                    if verbose {
                        s.push(format!("flags = 0x{:04x} ({})\n", flags, flags));
                    }

                    // Compression
                    let compression = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "compression = 0x{:04x} ({})\n",
                            compression, compression
                        ));
                    }

                    // Mod time
                    let (mod_time, mod_time_raw) = read_mod_time(r)?;
                    if verbose {
                        s.push(format!(
                            "mod_time = 0x{:04x} ({:?})\n",
                            mod_time_raw, mod_time
                        ));
                    }

                    // Mod date
                    let (mod_date, mod_date_raw) = read_mod_date(r)?;
                    if verbose {
                        s.push(format!(
                            "mod_date = 0x{:04x} ({:?})\n",
                            mod_date_raw, mod_date
                        ));
                    }

                    // CRC32
                    let crc32 = read_u32le(r)?;
                    if verbose {
                        s.push(format!("crc32 = 0x{:08x} ({})\n", crc32, crc32));
                    }

                    // Compressed size
                    let compressed_size = read_u32le(r)?;
                    if verbose {
                        s.push(format!(
                            "compressed_size = 0x{:08x} ({})\n",
                            compressed_size, compressed_size
                        ));
                    }

                    // Uncompressed size
                    let uncompressed_size = read_u32le(r)?;
                    if verbose {
                        s.push(format!(
                            "uncompressed_size = 0x{:08x} ({})\n",
                            uncompressed_size, uncompressed_size,
                        ));
                    }

                    // File name length
                    let file_name_length = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "file_name_length = 0x{:04x} ({})\n",
                            file_name_length, file_name_length
                        ));
                    }

                    // Extra field length
                    let extra_field_length = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "extra_field_length = 0x{:04x} ({})\n",
                            extra_field_length, extra_field_length,
                        ));
                    }

                    // File name
                    let file_name = read_n(file_name_length.into(), r)?;
                    if verbose {
                        s.push(format!(
                            "file_name = {:?} ({:?})\n",
                            hex::encode(&file_name),
                            std::str::from_utf8(&file_name).unwrap(),
                        ));
                    }

                    // Extra field
                    let extra_field = read_n(extra_field_length.into(), r)?;
                    if verbose {
                        s.push(format!("extra_field = {:?}\n", hex::encode(&extra_field)));
                    }

                    // File data
                    let file_data = read_n(compressed_size as usize, r)?;
                    if verbose {
                        s.push(format!("file_data = {:?}\n", hex::encode(&file_data)));
                    }

                    // Data descriptor
                    let data_descriptor = if flags & (1 << 3) != 0 {
                        let crc32 = read_u32le(r)?;
                        let compressed_size = read_u32le(r)?;
                        let uncompressed_size = read_u32le(r)?;
                        Some((crc32, compressed_size, uncompressed_size))
                    } else {
                        None
                    };
                    if verbose {
                        s.push(format!("data_descriptor = {:?}\n", data_descriptor));
                    }

                // Central directory file header
                } else if sig == 0x504b0102 {
                    if verbose {
                        s.push(format!(
                            "sig = 0x{:08x} (Central directory file header)\n",
                            sig
                        ));
                    }

                    // Version
                    let version = read_u16le(r)?;
                    if verbose {
                        s.push(format!("version = 0x{:04x} ({})\n", version, version));
                    }

                    // Version needed
                    let version_needed = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "version_needed = 0x{:04x} ({})\n",
                            version_needed, version_needed
                        ));
                    }

                    // Flags
                    let flags = read_u16le(r)?;
                    if verbose {
                        s.push(format!("flags = 0x{:04x} ({})\n", flags, flags));
                    }

                    // Compression
                    let compression = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "compression = 0x{:04x} ({})\n",
                            compression, compression
                        ));
                    }

                    // Mod time
                    let (mod_time, mod_time_raw) = read_mod_time(r)?;
                    if verbose {
                        s.push(format!(
                            "mod_time = 0x{:04x} ({:?})\n",
                            mod_time_raw, mod_time
                        ));
                    }

                    // Mod date
                    let (mod_date, mod_date_raw) = read_mod_date(r)?;
                    if verbose {
                        s.push(format!(
                            "mod_date = 0x{:04x} ({:?})\n",
                            mod_date_raw, mod_date
                        ));
                    }

                    // CRC32
                    let crc32 = read_u32le(r)?;
                    if verbose {
                        s.push(format!("crc32 = 0x{:08x} ({})\n", crc32, crc32));
                    }

                    // Compressed size
                    let compressed_size = read_u32le(r)?;
                    if verbose {
                        s.push(format!(
                            "compressed_size = 0x{:08x} ({})\n",
                            compressed_size, compressed_size
                        ));
                    }

                    // Uncompressed size
                    let uncompressed_size = read_u32le(r)?;
                    if verbose {
                        s.push(format!(
                            "uncompressed_size = 0x{:08x} ({})\n",
                            uncompressed_size, uncompressed_size,
                        ));
                    }

                    // File name length
                    let file_name_length = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "file_name_length = 0x{:04x} ({})\n",
                            file_name_length, file_name_length
                        ));
                    }

                    // Extra field length
                    let extra_field_length = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "extra_field_length = 0x{:04x} ({})\n",
                            extra_field_length, extra_field_length,
                        ));
                    }

                    // File comment length
                    let file_comment_length = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "file_comment_length = 0x{:04x} ({})\n",
                            file_comment_length, file_comment_length,
                        ));
                    }

                    // Disk # start
                    let disk_number_start = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "disk_number_start = 0x{:04x} ({})\n",
                            disk_number_start, disk_number_start,
                        ));
                    }

                    // Internal file attributes
                    let internal_file_attributes = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "internal_file_attributes = 0x{:04x} ({})\n",
                            internal_file_attributes, internal_file_attributes,
                        ));
                    }

                    // External file attributes
                    let external_file_attributes = read_u32le(r)?;
                    if verbose {
                        s.push(format!(
                            "external_file_attributes = 0x{:08x} ({})\n",
                            external_file_attributes, external_file_attributes,
                        ));
                    }

                    // Offset of local file header
                    let lfh_offset = read_u32le(r)?;
                    if verbose {
                        s.push(format!(
                            "lfh_offset = 0x{:08x} ({})\n",
                            lfh_offset, lfh_offset,
                        ));
                    }

                    // File name
                    let file_name = read_n(file_name_length.into(), r)?;
                    if verbose {
                        s.push(format!(
                            "file_name = {:?} ({:?})\n",
                            hex::encode(&file_name),
                            std::str::from_utf8(&file_name).unwrap(),
                        ));
                    }

                    // Extra field
                    let extra_field = read_n(extra_field_length.into(), r)?;
                    if verbose {
                        s.push(format!("extra_field = {:?}\n", hex::encode(&extra_field)));
                    }

                    // File comment
                    let file_comment = read_n(file_comment_length as usize, r)?;
                    if verbose {
                        s.push(format!(
                            "file_comment = {:?} ({:?})\n",
                            hex::encode(&file_comment),
                            std::str::from_utf8(&file_comment).unwrap(),
                        ));
                    }

                    // Summary
                    if !verbose {
                        s.push(format!(
                            "\
{}\t{}\t{uncompressed_size}\t{:04}-{:02}-{:02}T{:02}:{:02}:{:02}\t{}
\
                            ",
                            std::str::from_utf8(&file_name).unwrap(),
                            file_name.ends_with(b"/"),
                            mod_date.0,
                            mod_date.1,
                            mod_date.2,
                            mod_time.0,
                            mod_time.1,
                            mod_time.2,
                            std::str::from_utf8(&file_comment).unwrap(),
                        ));
                    }

                // End of central directory record
                } else if sig == 0x504b0506 {
                    if verbose {
                        s.push(format!(
                            "sig = 0x{:08x} (End of central directory record)\n",
                            sig
                        ));
                    }

                    // Disk number
                    let disk_number = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "disk_number = 0x{:04x} ({})\n",
                            disk_number, disk_number,
                        ));
                    }

                    // Disk number w/ central directory
                    let disk_number_w_cd = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "disk_number_w_cd = 0x{:04x} ({})\n",
                            disk_number_w_cd, disk_number_w_cd,
                        ));
                    }

                    // Disk entries
                    let disk_entries = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "disk_entries = 0x{:04x} ({})\n",
                            disk_entries, disk_entries,
                        ));
                    }

                    let total_entries = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "total_entries = 0x{:04x} ({})\n",
                            total_entries, total_entries,
                        ));
                    }

                    let cd_size = read_u32le(r)?;
                    if verbose {
                        s.push(format!("cd_size = 0x{:08x} ({})\n", cd_size, cd_size,));
                    }

                    let cd_offset = read_u32le(r)?;
                    if verbose {
                        s.push(format!("cd_offset = 0x{:08x} ({})\n", cd_offset, cd_offset,));
                    }

                    // Comment length
                    let comment_length = read_u16le(r)?;
                    if verbose {
                        s.push(format!(
                            "comment_length = 0x{:04x} ({})\n",
                            comment_length, comment_length,
                        ));
                    }
                } else {
                    return Err(format!("Invalid signature: `{:08x}`", sig));
                }
            }
            Err(_e) => {
                if verbose {
                    s.push(format!("EOF\n---\n"));
                }
                break;
            }
        }
    }

    let s = s.join("");

    if s == "---\nEOF\n---\n" {
        return Err(String::from("Unexpected end of file"));
    }

    Ok(s)
}

/// Process a zip file at path
fn process_file(path: &str, verbose: bool) -> Result<String, String> {
    if !Path::new(path).exists() {
        return Err(format!("No such file or directory (os error 2): {:?}", path));
    } else if !Path::new(path).is_file() {
        return Err(format!("Path is not a file: {:?}", path));
    }
    match File::open(path) {
        Ok(f) => {
            let mut r = BufReader::new(f);
            process(&mut r, verbose)
        }
        Err(e) => Err(format!("{e}: {:?}", path)),
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Field enum

    #[test]
    fn field_u16le_length() {
        assert_eq!(Field::U16LE.len(), 2);
    }

    #[test]
    fn field_u32le_length() {
        assert_eq!(Field::U32LE.len(), 4);
    }

    #[test]
    fn field_u32be_length() {
        assert_eq!(Field::U32BE.len(), 4);
    }

    // Conversion functions

    #[test]
    fn mod_time_test() {
        let bytes = hex::decode("b348").unwrap();
        assert_eq!(mod_time(&bytes), ((9, 5, 38), 0x48b3));
    }

    #[test]
    fn mod_date_test() {
        let bytes = hex::decode("1951").unwrap();
        assert_eq!(mod_date(&bytes), ((2020, 8, 25), 0x5119));
    }

    #[test]
    fn u16le_test() {
        let bytes = hex::decode("0100").unwrap();
        assert_eq!(u16le(&bytes), 1);
    }

    #[test]
    fn u32le_test() {
        let bytes = hex::decode("01000000").unwrap();
        assert_eq!(u32le(&bytes), 1);
    }

    #[test]
    fn u32be_test() {
        let bytes = hex::decode("00000001").unwrap();
        assert_eq!(u32be(&bytes), 1);
    }

    // Read functions

    #[test]
    fn read_n_test() {
        let bytes = hex::decode("00010203040506").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_n(3, &mut reader);
        assert_eq!(v, Ok(vec![0, 1, 2]));
    }

    #[test]
    fn read_n_test_fail() {
        let bytes = hex::decode("00010203040506").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_n(8, &mut reader);
        assert_eq!(v, Err(String::from("failed to fill whole buffer")));
    }

    #[test]
    fn read_test() {
        let bytes = hex::decode("00010203040506").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read(Field::U16LE, &mut reader);
        assert_eq!(v, Ok(vec![0, 1]));
        let v = read(Field::U32LE, &mut reader);
        assert_eq!(v, Ok(vec![2, 3, 4, 5]));
        let v = read(Field::U32BE, &mut reader);
        assert_eq!(v, Err(String::from("failed to fill whole buffer")));
    }

    #[test]
    fn read_mod_time_test() {
        let bytes = hex::decode("b348").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_mod_time(&mut reader);
        assert_eq!(v, Ok(((9, 5, 38), 0x48b3)));
    }

    #[test]
    fn read_mod_date_test() {
        let bytes = hex::decode("1951").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_mod_date(&mut reader);
        assert_eq!(v, Ok(((2020, 8, 25), 0x5119)));
    }

    #[test]
    fn read_u16le_test() {
        let bytes = hex::decode("0100").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_u16le(&mut reader);
        assert_eq!(v, Ok(1));
    }

    #[test]
    fn read_u32le_test() {
        let bytes = hex::decode("01000000").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_u32le(&mut reader);
        assert_eq!(v, Ok(1));
    }

    #[test]
    fn read_u32be_test() {
        let bytes = hex::decode("00000001").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_u32be(&mut reader);
        assert_eq!(v, Ok(1));
    }

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
        assert_eq!(
            process(&mut r, true).unwrap(),
            include_str!("../../exercise.zip-process-verbose.txt"),
        );
    }

    #[test]
    fn process_summary_test() {
        let f = File::open("../../exercise.zip").unwrap();
        let mut r = BufReader::new(f);
        assert_eq!(
            process(&mut r, false).unwrap(),
            include_str!("../../exercise.zip-process-summary.txt"),
        );
    }

    #[test]
    fn process_file_nonexistent_test() {
        assert_eq!(
            process_file("../../nonexistent.zip", true).unwrap_err(),
            String::from("No such file or directory (os error 2): \"../../nonexistent.zip\""),
        );
    }

    #[test]
    fn process_file_not_file_test() {
        assert_eq!(
            process_file(".", true).unwrap_err(),
            String::from("Path is not a file: \".\""),
        );
    }

    #[test]
    fn process_file_test() {
        assert_eq!(
            process_file("../../exercise.zip", true).unwrap(),
            include_str!("../../exercise.zip-process-verbose.txt"),
        );
    }

    #[test]
    fn process_file_summary_test() {
        assert_eq!(
            process_file("../../exercise.zip", false).unwrap(),
            include_str!("../../exercise.zip-process-summary.txt"),
        );
    }
}
