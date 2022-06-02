use crate::*;

#[derive(BinRead, Debug)]
pub struct Entries {
    #[br(parse_with = until_eof)]
    pub list: Vec<Entry>,
}

#[derive(BinRead, Debug)]
pub enum Entry {
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
    pub fn verbose(&self) -> String {
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
    pub fn verbose(&self) -> String {
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
    pub fn verbose(&self) -> String {
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

    pub fn summary(&self) -> String {
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
    pub fn verbose(&self) -> String {
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
