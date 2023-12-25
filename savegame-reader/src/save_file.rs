use byteorder::ReadBytesExt;
use std::io;
use std::io::{Read, Seek, SeekFrom};

use crate::common::has_bit;
use xz2::read::XzDecoder;

pub trait SaveFile: Read {
    fn debug_info(&mut self) -> String;

    fn read_gamma(&mut self) -> io::Result<usize> {
        let mut length = usize::from(self.read_u8()?);
        if has_bit(length, 7) {
            length &= !0b1000_0000;
            if has_bit(length, 6) {
                length &= !0b0100_0000;
                if has_bit(length, 5) {
                    length &= !0b0010_0000;
                    if has_bit(length, 4) {
                        length &= !0b0001_0000;
                        if has_bit(length, 3) {
                            panic!("Unsupported array length")
                        }
                        length = length << 8 | usize::from(self.read_u8()?);
                    }
                    length = length << 8 | usize::from(self.read_u8()?);
                }
                length = length << 8 | usize::from(self.read_u8()?);
            }
            length = length << 8 | usize::from(self.read_u8()?);
        }
        Ok(length)
    }

    fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_gamma()?;
        let mut buf = vec![0; length];
        self.read_exact(&mut buf)?;

        Ok(String::from(std::str::from_utf8(&buf).unwrap()))
    }
}

pub struct CompressedSaveFile<'a> {
    pub version: u32,
    pub reader: Box<dyn Read + 'a>,
}

impl<'a> CompressedSaveFile<'a> {
    pub fn new<R: Read + 'a>(mut reader: R) -> CompressedSaveFile<'a> {
        let format = SaveFileFormat::read_from(&mut reader);

        let mut version = [0; 4];
        reader.read_exact(&mut version).unwrap();
        let version = u32::from_be_bytes(version) >> 16;

        let reader = match format {
            SaveFileFormat::Lzma => XzDecoder::new(reader),
            format => panic!("Unsupported savegame format: {:?}", format),
        };

        CompressedSaveFile {
            version,
            reader: Box::new(reader),
        }
    }

    pub fn debug_info(&mut self) -> String {
        String::from("No debug info available")
    }
}

impl<'a> Read for CompressedSaveFile<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<'a> SaveFile for CompressedSaveFile<'a> {
    fn debug_info(&mut self) -> String {
        String::from("No debug info available")
    }
}

pub struct DebugSaveFile<R: Read + Seek> {
    pub reader: Box<R>,
}

impl<R: Read + Seek> DebugSaveFile<R> {
    pub fn new_from_decoded(reader: R) -> DebugSaveFile<R> {
        DebugSaveFile {
            reader: Box::new(reader),
        }
    }

    pub fn debug_info(&mut self) -> String {
        format!(
            "Current position: {}",
            self.reader.stream_position().unwrap()
        )
    }
}

impl<R: Read + Seek> Read for DebugSaveFile<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<R: Read + Seek> Seek for DebugSaveFile<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

impl<R: Read + Seek> SaveFile for DebugSaveFile<R> {
    fn debug_info(&mut self) -> String {
        format!(
            "Position in file: {}",
            self.reader.stream_position().unwrap()
        )
    }
}

#[derive(Debug)]
enum SaveFileFormat {
    Lzo,
    Zlib,
    Lzma,
    Unknown,
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
            _ => SaveFileFormat::Unknown,
        }
    }
}
