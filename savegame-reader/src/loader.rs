use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use byteorder::ReadBytesExt;
use xz2::read::XzDecoder;

pub fn load_file(path: &Path) {
    let mut file = File::open(path).unwrap();
    check_version_support(&mut file);

    let mut decoder = XzDecoder::new(file);

    let mut chunk_id = [0; 4];
    decoder.read_exact(&mut chunk_id).unwrap();

    println!("Loading chunk {}", chunk_id_from_bytes(&chunk_id));
    let chunk_type = ChunkType::read_from(&mut decoder);
    println!("Chunk type: {:?}", chunk_type);

    if chunk_type.has_table_header() {
        // SlIterateArray
        // read array length
        let array_length = read_array_length(&mut decoder);
        println!("Length: {}", array_length);
        let fields = read_table_header(&mut decoder);
        println!("Fields: {:?}", fields)
    }
}

fn read_table_header(reader: &mut impl Read) -> Vec<Field> {
    let mut fields = vec![];
    loop {
        let data_type = reader.read_u8().unwrap();
        if data_type == 0 {
            break;
        }

        let key = read_str(reader);
        fields.push(Field { key, data_type });
    }
    fields
}

#[derive(Debug)]
struct Field {
    key: String,
    data_type: u8,
}

fn read_conv(reader: &mut impl Read) {
    reader.read_u8().unwrap();
}

fn read_str(reader: &mut impl Read) -> String {
    let length = read_array_length(reader);
    let mut buf = vec![0; length];
    reader.read_exact(&mut buf).unwrap();

    String::from(std::str::from_utf8(&buf).unwrap())
}

fn get_bit_at(input: usize, n: u8) -> bool {
    if n < 32 {
        input & (1 << n) != 0
    } else {
        false
    }
}

fn read_array_length(reader: &mut impl Read) -> usize {
    let mut length = usize::from(reader.read_u8().unwrap());
    if get_bit_at(length, 7) {
        length &= !0b1000_0000;
        if get_bit_at(length, 6) {
            length &= !0b0100_0000;
            if get_bit_at(length, 5) {
                length &= !0b0010_0000;
                if get_bit_at(length, 4) {
                    length &= !0b0001_0000;
                    if get_bit_at(length, 3) {
                        panic!("Unsupported array length")
                    }
                    length = length << 8 | usize::from(reader.read_u8().unwrap());
                }
                length = length << 8 | usize::from(reader.read_u8().unwrap());
            }
            length = length << 8 | usize::from(reader.read_u8().unwrap());
        }
        length = length << 8 | usize::from(reader.read_u8().unwrap());
    }
    length
}

#[derive(Debug)]
enum ChunkType {
    Riff,
    Array,
    SparseArray,
    Table,
    SparseTable,
}

impl ChunkType {
    fn read_from(reader: &mut impl Read) -> ChunkType {
        const TYPE_MASK: u8 = 0xF;
        match reader.read_u8().unwrap() & TYPE_MASK {
            0 => ChunkType::Riff,
            1 => ChunkType::Array,
            2 => ChunkType::SparseArray,
            3 => ChunkType::Table,
            4 => ChunkType::SparseTable,
            _ => panic!("Unknown chunk type")
        }
    }

    fn has_table_header(&self) -> bool {
        match self {
            ChunkType::Table | ChunkType::SparseTable => true,
            _ => false
        }
    }
}

fn chunk_id_from_bytes(bytes: &[u8; 4]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

fn check_version_support(file: &mut impl Read) {
    match SaveFileFormat::read_from(file) {
        SaveFileFormat::Lzma => (),
        _ => panic!("Unsupported savegame format")
    }

    let mut version = [0; 4];
    file.read_exact(&mut version).unwrap();
    let version = u32::from_be_bytes(version) >> 16;
    println!("Savegame version {}", version)
}

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
            _ => SaveFileFormat::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::loader::load_file;

    #[test]
    fn testy() {
        load_file(Path::new("/Users/andreas/Projects/personal/openttd-viz/savegame-reader/test.sav"))
    }
}