use std::io::{BufReader, Read};

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
    let n = a.iter().take(2).enumerate().map(|(i, x)| (*x as u16) << (8 * i)).sum();
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
    let n = a.iter().take(2).enumerate().map(|(i, x)| (*x as u16) << (8 * i)).sum();
    let y = (n >> 9) + 1980u16;
    let m = ((n >> 5) & (0b0000000000001111 as u16)) as u8;
    let d = (n & (0b0000000000011111 as u16)) as u8;
    ((y, m, d), n)
}

/// Convert 2 u8's (little endian) into a u16
fn u16le(a: &[u8]) -> u16 {
    a.iter().take(2).enumerate().map(|(i, x)| (*x as u16) << (8 * i)).sum()
}

/// Convert 4 u8's (little endian) into a u32
fn u32le(a: &[u8]) -> u32 {
    a.iter().take(4).enumerate().map(|(i, x)| (*x as u32) << (8 * i)).sum()
}

/// Convert 4 u8's (big endian) into a u32
fn u32be(a: &[u8]) -> u32 {
    a.iter().take(4).rev().enumerate().map(|(i, x)| (*x as u32) << (8 * i)).sum()
}

// Read functions

/// Read `n` bytes from a `BufReader`
fn read_n<R>(n: usize, r: &mut BufReader<R>, path: &str) -> Result<Vec<u8>, String>
where R: Read
{
    let mut buf = vec![0; n];
    match r.read_exact(&mut buf) {
        Ok(()) => Ok(buf),
        Err(e) => Err(format!("{}: {:?}", e, path)),
    }
}

/// Read correct number of bytes for a `Field` from a `BufReader`
fn read<R>(field: Field, r: &mut BufReader<R>, path: &str) -> Result<Vec<u8>, String>
where R: Read
{
    read_n(field.len(), r, path)
}

/// Read a file modification time from a `BufReader`
fn read_mod_time<R>(r: &mut BufReader<R>, path: &str) -> Result<((u8, u8, u8), u16), String>
where R: Read
{
    Ok(mod_time(&read(Field::U16LE, r, path)?))
}

/// Read a file modification date from a `BufReader`
fn read_mod_date<R>(r: &mut BufReader<R>, path: &str) -> Result<((u16, u8, u8), u16), String>
where R: Read
{
    Ok(mod_date(&read(Field::U16LE, r, path)?))
}

/// Read a u16 (little endian) from a `BufReader`
fn read_u16le<R>(r: &mut BufReader<R>, path: &str) -> Result<u16, String>
where R: Read
{
    Ok(u16le(&read(Field::U16LE, r, path)?))
}

/// Read a u32 (little endian) from a `BufReader`
fn read_u32le<R>(r: &mut BufReader<R>, path: &str) -> Result<u32, String>
where R: Read
{
    Ok(u32le(&read(Field::U32LE, r, path)?))
}

/// Read a u32 (big endian) from a `BufReader`
fn read_u32be<R>(r: &mut BufReader<R>, path: &str) -> Result<u32, String>
where R: Read
{
    Ok(u32be(&read(Field::U32BE, r, path)?))
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
        let v = read_n(3, &mut reader, "test");
        assert_eq!(v, Ok(vec![0, 1, 2]));
    }

    #[test]
    fn read_n_test_fail() {
        let bytes = hex::decode("00010203040506").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_n(8, &mut reader, "test");
        assert_eq!(v, Err(String::from("failed to fill whole buffer: \"test\"")));
    }

    #[test]
    fn read_test() {
        let bytes = hex::decode("00010203040506").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read(Field::U16LE, &mut reader, "test");
        assert_eq!(v, Ok(vec![0, 1]));
        let v = read(Field::U32LE, &mut reader, "test");
        assert_eq!(v, Ok(vec![2, 3, 4, 5]));
        let v = read(Field::U32BE, &mut reader, "test");
        assert_eq!(v, Err(String::from("failed to fill whole buffer: \"test\"")));
    }

    #[test]
    fn read_mod_time_test() {
        let bytes = hex::decode("b348").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_mod_time(&mut reader, "test");
        assert_eq!(v, Ok(((9, 5, 38), 0x48b3)));
    }

    #[test]
    fn read_mod_date_test() {
        let bytes = hex::decode("1951").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_mod_date(&mut reader, "test");
        assert_eq!(v, Ok(((2020, 8, 25), 0x5119)));
    }

    #[test]
    fn read_u16le_test() {
        let bytes = hex::decode("0100").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_u16le(&mut reader, "test");
        assert_eq!(v, Ok(1));
    }

    #[test]
    fn read_u32le_test() {
        let bytes = hex::decode("01000000").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_u32le(&mut reader, "test");
        assert_eq!(v, Ok(1));
    }

    #[test]
    fn read_u32be_test() {
        let bytes = hex::decode("00000001").unwrap();
        let cursor = Cursor::new(&bytes);
        let mut reader = BufReader::new(cursor);
        let v = read_u32be(&mut reader, "test");
        assert_eq!(v, Ok(1));
    }
}
