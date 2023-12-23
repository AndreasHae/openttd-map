use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use crate::base_readers::read_gamma;
use crate::header::SaveFileHeader;
use crate::table_reader::{read_table_header, TableItem};

#[allow(dead_code)]
pub fn load_file(path: &Path) {
    let mut file = File::open(path).unwrap();
    let header = SaveFileHeader::read_from(&mut file);
    println!("{:?}", header);

    let mut decoder = header.format.get_decoder(file);

    let mut chunks: HashMap<String, Vec<TableItem>> = HashMap::new();

    loop {
        let mut chunk_id = [0; 4];
        decoder.read_exact(&mut chunk_id).unwrap();
        if u32::from_be_bytes(chunk_id) == 0 {
            break;
        }

        let chunk_id = chunk_id_from_bytes(&chunk_id);
        println!("Loading chunk {}", chunk_id);
        let chunk_type = ChunkType::read_from(&mut decoder);
        println!("Chunk type: {:?}", chunk_type);

        if chunk_type.has_table_header() {
            // SlIterateArray
            // read array length
            let table_header_length = read_gamma(&mut decoder);
            assert!(table_header_length > 0, "table header size was 0");
            println!("Table header length: {} bytes", table_header_length);
            let fields = read_table_header(&mut decoder);
            // println!("Fields in header: {:#?}", fields);

            let mut parsed_items: Vec<TableItem> = Vec::new();

            if chunk_type == ChunkType::Table {
                let mut index = 0usize;
                loop {
                    let mut size = read_gamma(&mut decoder);
                    if size == 0 {
                        break;
                    }
                    size -= 1;

                    println!("Object length: {} bytes", size);
                    index += 1;
                    println!("Index: {}", index);

                    let parsed_fields: TableItem = TableItem(
                        fields
                            .iter()
                            .map(|field| field.parse_from(&mut decoder))
                            .collect(),
                    );
                    parsed_items.push(parsed_fields);
                }
            }

            if chunk_type == ChunkType::SparseTable {
                loop {
                    let mut obj_length = read_gamma(&mut decoder);
                    if obj_length == 0 {
                        break;
                    }
                    obj_length -= 2;

                    println!("Object length: {} bytes", obj_length);
                    let index = read_gamma(&mut decoder);
                    println!("Index: {}", index);
                    skip_bytes(&mut decoder, obj_length);
                }
            }

            if parsed_items.len() != 0 {
                chunks.insert(String::from(chunk_id), parsed_items);
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

    println!("{}", serde_json::to_string(&chunks).unwrap())
}

fn skip_bytes(decoder: &mut impl Read, bytes: usize) {
    let mut buf = vec![0; bytes];
    decoder.read_exact(&mut buf).unwrap();
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::loader::load_file;

    #[test]
    fn testy() {
        load_file(Path::new("./test-big.sav"))
    }
}
