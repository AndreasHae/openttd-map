use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;

use crate::base_readers::{has_bit, read_gamma, read_str};

#[derive(Serialize)]
pub struct TableItem(pub Vec<ParsedField>);

#[derive(Debug, Serialize)]
pub struct ParsedField {
    key: String,
    data: ParsedFieldData,
}

#[derive(Debug, Serialize)]
pub enum ParsedFieldData {
    Scalar(ParsedFieldContent),
    List(Vec<ParsedFieldContent>),
}

#[derive(Debug, Serialize)]
pub enum ParsedFieldContent {
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

pub fn read_table(
    mut decoder: &mut impl Read,
    fields: Vec<Field>,
) -> Vec<TableItem> {
    let mut index = 0usize;
    let mut parsed_items: Vec<TableItem> = Vec::new();
    loop {
        let mut size = read_gamma(&mut decoder);
        if size == 0 {
            break;
        }
        size -= 1;

        // println!("Object length: {} bytes", size);
        index += 1;
        // println!("Index: {}", index);

        let parsed_fields: TableItem = TableItem(
            fields
                .iter()
                .map(|field| field.parse_from(&mut decoder))
                .collect(),
        );
        parsed_items.push(parsed_fields);
    }
    parsed_items
}

#[derive(Debug)]
pub struct Field {
    key: String,
    var_type: VarType,
    children: Option<Vec<Field>>,
}

impl Field {
    pub fn parse_from(&self, reader: &mut impl Read) -> ParsedField {
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

pub fn read_table_header(reader: &mut impl Read) -> Vec<Field> {
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
