use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use crate::base_readers::{has_bit, read_gamma, read_str};
use crate::header::SaveFileHeader;

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

#[derive(Serialize)]
struct TableItem(Vec<ParsedField>);

#[derive(Debug, Serialize)]
struct ParsedField {
    key: String,
    data: ParsedFieldData,
}

#[derive(Debug, Serialize)]
enum ParsedFieldData {
    Scalar(ParsedFieldContent),
    List(Vec<ParsedFieldContent>),
}

#[derive(Debug, Serialize)]
enum ParsedFieldContent {
    FileEnd,
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    StringId,
    String,
    Struct(Vec<ParsedField>),
}

fn skip_bytes(decoder: &mut impl Read, bytes: usize) {
    let mut buf = vec![0; bytes];
    decoder.read_exact(&mut buf).unwrap();
}

fn read_table_header(reader: &mut impl Read) -> Vec<Field> {
    let mut fields = vec![];
    loop {
        let var_type = VarType::from_byte(reader.read_u8().unwrap());
        match var_type {
            None => break,
            Some(var_type) => {
                let key = read_str(reader);
                fields.push(Field {
                    key,
                    var_type,
                    children: None,
                });
            }
        }
    }
    for field in fields.as_mut_slice() {
        if field.var_type.data_type() == DataType::Struct {
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

impl Field {
    fn parse_from(&self, reader: &mut impl Read) -> ParsedField {
        let data = match &self.var_type {
            VarType::List(data_type) => {
                let length = read_gamma(reader);
                let mut items = vec![];
                for i in 0..length {
                    let value = if let Some(children) = &self.children {
                        let mut parsed_children = vec![];
                        for child_field in children {
                            parsed_children.push(child_field.parse_from(reader));
                        }
                        ParsedFieldContent::Struct(parsed_children)
                    } else {
                        data_type.read_from(reader)
                    };
                    items.push(value);
                }
                ParsedFieldData::List(items)
            }
            VarType::Scalar(data_type) => {
                let content = if let Some(children) = &self.children {
                    let mut parsed_children: Vec<ParsedField> = vec![];
                    for child_field in children {
                        parsed_children.push(child_field.parse_from(reader));
                    }
                    ParsedFieldContent::Struct(parsed_children)
                } else {
                    data_type.read_from(reader)
                };
                ParsedFieldData::Scalar(content)
            }
        };

        ParsedField {
            key: self.key.clone(),
            data,
        }
    }
}

#[derive(Debug, Clone)]
enum VarType {
    Scalar(DataType),
    List(DataType),
}

impl VarType {
    fn from_byte(byte: u8) -> Option<VarType> {
        if byte == 0 {
            return None;
        }

        let data_type = DataType::from_byte(byte & 0b0000_1111);
        let has_length_field = has_bit(usize::from(byte), 4);

        if has_length_field {
            Some(VarType::List(data_type))
        } else {
            Some(VarType::Scalar(data_type))
        }
    }

    fn data_type(&self) -> DataType {
        match self {
            VarType::Scalar(data_type) => data_type.clone(),
            VarType::List(data_type) => data_type.clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum DataType {
    FileEnd,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    StringId,
    String,
    Struct,
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

    fn read_from(&self, reader: &mut impl Read) -> ParsedFieldContent {
        match self {
            DataType::I8 => ParsedFieldContent::I8(reader.read_i8().unwrap()),
            DataType::U8 => ParsedFieldContent::U8(reader.read_u8().unwrap()),
            DataType::I16 => ParsedFieldContent::I16(reader.read_i16::<BigEndian>().unwrap()),
            DataType::U16 => ParsedFieldContent::U16(reader.read_u16::<BigEndian>().unwrap()),
            DataType::I32 => ParsedFieldContent::I32(reader.read_i32::<BigEndian>().unwrap()),
            DataType::U32 => ParsedFieldContent::U32(reader.read_u32::<BigEndian>().unwrap()),
            DataType::I64 => ParsedFieldContent::I64(reader.read_i64::<BigEndian>().unwrap()),
            DataType::U64 => ParsedFieldContent::U64(reader.read_u64::<BigEndian>().unwrap()),
            _ => panic!(),
        }
    }
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
