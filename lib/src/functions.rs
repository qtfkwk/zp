use crate::*;

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
pub fn mod_time(n: u16) -> ((u8, u8, u8), u16) {
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
pub fn mod_date(n: u16) -> ((u16, u8, u8), u16) {
    let y = (n >> 9) + 1980u16;
    let m = ((n >> 5) & (0b0000000000001111 as u16)) as u8;
    let d = (n & (0b0000000000011111 as u16)) as u8;
    ((y, m, d), n)
}

/// Convert a `binrw::Error::BadMagic.found` (`[0, 1, 2, 3]`) into a nice hex string (`00010203`)
pub fn magic_hex(magic: &str) -> String {
    magic
        .replace("[", "")
        .replace("]", "")
        .split(", ")
        .map(|x| format!("{:02x}", x.parse::<u8>().unwrap()))
        .collect::<Vec<String>>()
        .join("")
}
