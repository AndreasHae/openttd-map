use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};

use crate::base_readers::read_gamma;
use crate::header::SaveFileHeader;
use crate::table_reader::{read_sparse_table, read_table, read_table_header, TableItem};

#[allow(dead_code)]
pub fn load_file(path: &Path) {
    let mut file = File::open(path).unwrap();
    let header = SaveFileHeader::read_from(&mut file);
    println!("{:?}", header);

    // let mut decoder = header.format.get_decoder(file);
    let mut decoder = File::open("test-big.sav.decoded").unwrap();

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
            // println!("Table header length: {} bytes", table_header_length);
            let fields = read_table_header(&mut decoder);
            // println!("Fields in header: {:#?}", fields);

            if chunk_id == "ORDR" {
                println!("Fields in header: {:#?}", fields);
            }

            match chunk_type {
                ChunkType::Table => {
                    if chunk_id == "ORDR" {
                        println!("{}", decoder.seek(SeekFrom::Current(0)).unwrap());
                    }
                    let items = read_table(&mut decoder, fields);
                    chunks.insert(String::from(chunk_id), items);
                }
                ChunkType::SparseTable => {
                    let items = read_sparse_table(&mut decoder, fields);
                    chunks.insert(String::from(chunk_id), items);
                }
                _ => panic!("unexpected chunk type {:?}", chunk_type),
            }
        }
        if chunk_type == ChunkType::Riff {
            let mut length = usize::from(decoder.read_u8().unwrap()) << 16;
            length += usize::from(decoder.read_u16::<BigEndian>().unwrap());
            println!("Riff length: {}", length);
            skip_bytes(&mut decoder, length);
        }
        println!();
    }

    // println!("{}", serde_json::to_string(&chunks).unwrap())
    println!("{:?}", chunks.keys());
    // File::create(Path::new("./out.json")).unwrap().write(serde_json::to_string(&chunks.keys().collect()).unwrap().as_bytes());
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
