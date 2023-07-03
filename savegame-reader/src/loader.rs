use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::loader::ChunkType::{Riff, SparseTable, Table};
use byteorder::{BigEndian, ReadBytesExt};
use xz2::read::XzDecoder;

pub fn load_file(path: &Path) {
    let mut file = File::open(path).unwrap();
    check_version_support(&mut file);

    let mut decoder = XzDecoder::new(file);

    loop {
        let mut chunk_id = [0; 4];
        decoder.read_exact(&mut chunk_id).unwrap();

        if u32::from_be_bytes(chunk_id) == 0 {
            break;
        }

        println!("Loading chunk {}", chunk_id_from_bytes(&chunk_id));
        let chunk_type = ChunkType::read_from(&mut decoder);
        println!("Chunk type: {:?}", chunk_type);

        if chunk_type.has_table_header() {
            // SlIterateArray
            // read array length
            let table_header_length = read_array_length(&mut decoder);
            println!("Table header length: {} bytes", table_header_length);
            let fields = read_table_header(&mut decoder);
            // println!("Fields in header: {:#?}", fields);

            if chunk_type == Table {
                let obj_length = read_array_length(&mut decoder);
                println!("Object length: {} bytes", obj_length);
                skip_bytes(&mut decoder, obj_length);
            }

            if chunk_type == SparseTable {
                loop {
                    let mut obj_length = read_array_length(&mut decoder);
                    if obj_length == 0 {
                        break;
                    }
                    obj_length -= 2;

                    println!("Object length: {} bytes", obj_length);
                    let index = read_array_length(&mut decoder);
                    println!("Index: {}", index);
                    skip_bytes(&mut decoder, obj_length);
                }
            }
        }
        if chunk_type == ChunkType::Riff {
            let mut length = usize::from(decoder.read_u8().unwrap()) << 16;
            length += usize::from(decoder.read_u16::<BigEndian>().unwrap());
            println!("Riff length: {}", length);
            skip_bytes(&mut decoder, length);
        }
        println!()
    }
}

fn skip_bytes(decoder: &mut impl Read, bytes: usize) {
    let mut buf = vec![0; bytes];
    decoder.read_exact(&mut buf).unwrap();
}

fn read_table_header(reader: &mut impl Read) -> Vec<Field> {
    let mut fields = vec![];
    loop {
        let var_type = VarType::from_byte(reader.read_u8().unwrap());
        if var_type.data_type == DataType::FileEnd {
            break;
        }

        let key = read_str(reader);
        fields.push(Field {
            key,
            var_type,
            children: None,
        });
    }
    for field in fields.as_mut_slice() {
        if field.var_type.data_type == DataType::Struct {
            drop(std::mem::replace(
                field,
                Field {
                    key: field.key.clone(),
                    var_type: field.var_type.clone(),
                    children: Some(read_table_header(reader)),
                },
            ))
        }
    }
    fields
}

#[derive(Debug)]
struct Field {
    key: String,
    var_type: VarType,
    children: Option<Vec<Field>>,
}

#[derive(Debug, Clone)]
struct VarType {
    data_type: DataType,
    has_length_field: bool,
}

impl VarType {
    fn from_byte(byte: u8) -> VarType {
        VarType {
            data_type: DataType::from_byte(byte & 0b0000_1111),
            has_length_field: get_bit_at(usize::from(byte), 4),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum DataType {
    FileEnd = 0,
    I8 = 1,
    U8 = 2,
    I16 = 3,
    U16 = 4,
    I32 = 5,
    U32 = 6,
    I64 = 7,
    U64 = 8,
    StringId = 9,
    String = 10,
    Struct = 11,
}

impl DataType {
    fn from_byte(byte: u8) -> DataType {
        match byte {
            0 => DataType::FileEnd,
            1 => DataType::I8,
            2 => DataType::U8,
            3 => DataType::I16,
            4 => DataType::U16,
            5 => DataType::I32,
            6 => DataType::U32,
            7 => DataType::I64,
            8 => DataType::U64,
            9 => DataType::StringId,
            10 => DataType::String,
            11 => DataType::Struct,
            _ => panic!("tried to map {} to DataType", byte),
        }
    }
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

#[derive(Debug, Eq, PartialEq)]
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
            _ => panic!("Unknown chunk type"),
        }
    }

    fn has_table_header(&self) -> bool {
        match self {
            ChunkType::Table | ChunkType::SparseTable => true,
            _ => false,
        }
    }
}

fn chunk_id_from_bytes(bytes: &[u8; 4]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

fn check_version_support(file: &mut impl Read) {
    match SaveFileFormat::read_from(file) {
        SaveFileFormat::Lzma => (),
        _ => panic!("Unsupported savegame format"),
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
            _ => SaveFileFormat::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::loader::load_file;

    #[test]
    fn testy() {
        load_file(Path::new(
            "/Users/andreas/Projects/personal/openttd-viz/savegame-reader/test.sav",
        ))
    }
}
