use std::io::{Read, Seek, SeekFrom};
use xz2::read::XzDecoder;

#[derive(Debug)]
pub struct SaveFileHeader {
    pub format: SaveFileFormat,
    pub version: u16,
}

impl SaveFileHeader {
    pub fn read_from(reader: &mut (impl Read + Seek)) -> SaveFileHeader {
        let format = SaveFileFormat::read_from(reader);

        let mut version = [0; 2];
        reader.read_exact(&mut version).unwrap();
        let version = u16::from_be_bytes(version);

        // Ignore 2 bytes as they are unused.
        reader.seek(SeekFrom::Current(2)).unwrap();

        SaveFileHeader { format, version }
    }
}

#[derive(Debug)]
pub enum SaveFileFormat {
    Lzo,
    Zlib,
    Lzma,
    Unknown(String),
}

impl SaveFileFormat {
    fn read_from(reader: &mut impl Read) -> SaveFileFormat {
        let mut format = [0; 4];
        reader.read_exact(&mut format).unwrap();

        SaveFileFormat::from_bytes(&format)
    }

    fn from_bytes(bytes: &[u8; 4]) -> SaveFileFormat {
        match std::str::from_utf8(bytes).unwrap() {
            "OTTD" => SaveFileFormat::Lzo,
            "OTTZ" => SaveFileFormat::Zlib,
            "OTTX" => SaveFileFormat::Lzma,
            unknown => SaveFileFormat::Unknown(String::from(unknown)),
        }
    }

    pub fn get_decoder(self: &SaveFileFormat, reader: impl Read + Seek) -> impl Read {
        match self {
            SaveFileFormat::Lzma => XzDecoder::new(reader),
            _ => panic!("Unsupported save file format: {:?}", self)
        }
    }
}
