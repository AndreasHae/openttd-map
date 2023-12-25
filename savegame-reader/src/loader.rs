use std::collections::HashMap;
use std::io;
use std::io::prelude::*;

use byteorder::{BigEndian, ReadBytesExt};

use crate::save_file::SaveFile;
use crate::table_reader::{read_sparse_table, read_table, read_table_header, Field, TableItem};

#[allow(dead_code)]
pub fn load_file(mut save_file: impl SaveFile) -> io::Result<HashMap<String, Vec<TableItem>>> {
    let mut chunks: HashMap<String, Vec<TableItem>> = HashMap::new();

    loop {
        let mut chunk_id = [0; 4];
        save_file.read_exact(&mut chunk_id)?;

        if u32::from_be_bytes(chunk_id) == 0 {
            break;
        }

        let chunk_id = chunk_id_from_bytes(&chunk_id);
        println!("Loading chunk {} ({})", chunk_id, chunk_name_of(chunk_id));
        let chunk_type = ChunkType::read_from(&mut save_file)?;
        println!("Chunk type: {:?}", chunk_type);
        println!("{}", save_file.debug_info());

        if chunk_type.has_table_header() {
            // SlIterateArray
            // read array length
            let table_header_length = save_file.read_gamma()?;
            assert!(table_header_length > 0, "table header size was 0");
            // println!("Table header length: {} bytes", table_header_length);
            let mut fields = read_table_header(&mut save_file)?;
            if chunk_id == "GSDT" {
                fields.push(Field::new_custom_data())
            }

            match chunk_type {
                ChunkType::Table => {
                    let items = read_table(&mut save_file, fields)?;
                    chunks.insert(String::from(chunk_id), items);
                }
                ChunkType::SparseTable => {
                    let items = read_sparse_table(&mut save_file, fields)?;
                    chunks.insert(String::from(chunk_id), items);
                }
                _ => panic!("unexpected chunk type {:?}", chunk_type),
            }
        }
        if chunk_type == ChunkType::Riff {
            let mut length = usize::from(save_file.read_u8()?) << 16;
            length += usize::from(save_file.read_u16::<BigEndian>()?);
            println!("Riff length: {}", length);
            skip_bytes(&mut save_file, length)?;
        }
        println!();
    }
    Ok(chunks)
}

fn skip_bytes(save_file: &mut impl Read, bytes: usize) -> io::Result<()> {
    let mut buf = vec![0; bytes];
    save_file.read_exact(&mut buf)
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
    fn read_from(reader: &mut impl Read) -> io::Result<ChunkType> {
        const TYPE_MASK: u8 = 0xF;
        Ok(match reader.read_u8()? & TYPE_MASK {
            0 => ChunkType::Riff,
            1 => ChunkType::Array,
            2 => ChunkType::SparseArray,
            3 => ChunkType::Table,
            4 => ChunkType::SparseTable,
            _ => panic!("Unknown chunk type"),
        })
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

fn chunk_name_of(chunk_id: &str) -> &str {
    match chunk_id {
        "AIPL" => "AI companies",
        "ANIT" => "Animated tiles",
        "ERNW" => "Engine renewal",
        "CMDL" => "Cargo deliveries",
        "CMPU" => "Cargo pickups",
        "CAPA" => "Cargo packets",
        "CHTS" => "Cheats used",
        "PLYR" => "Player companies",
        "DEPT" => "Depots",
        "PRIC" => "Prices",
        "CAPR" => "Cargo payment rates (only in pre 126 savegames)",
        "ECMY" => "Economy variables",
        "CAPY" => "Cargo payments",
        "ENGN" => "Engines",
        "ENGS" => "Engine ID mappings (legacy)",
        "EIDS" => "Engine ID mappings",
        "GSDT" => "Game script configs",
        "GSTR" => "Game strings",
        "GLOG" => "Game log",
        "GOAL" => "Goals",
        "GRPS" => "Groups (of vehicles?)",
        "INDY" => "Industries",
        "IBLD" => "Industry builder",
        "ITBL" => "Industry type build data",
        "RAIL" => "Rail type labels",
        "LEAE" => "League table elements",
        "LEAT" => "League tables",
        "LGRP" => "Link graphs",
        "LGRJ" => "Link graph jobs",
        "LGRS" => "Link graph schedule",
        "MAPS" => "Map size",
        "MAPT" => "Map tile types",
        "MAPH" => "Map tile heights",
        "MAPO" => "Map tile ownership information",
        "MAP2" => "Map tile indices of towns, industries and stations",
        "M3LO" => "Map tile general purpose",
        "M3HI" => "Map tile general purpose",
        "MAP5" => "Map tile general purpose",
        "MAPE" => "Map tile general purpose",
        "MAP7" => "Map tile NewGRF support",
        "MAP8" => "Map tile general purpose",
        "DATE" => "Ingame date",
        "VIEW" => "Viewport info",
        "NGRF" => "Loaded NewGRFs",
        "OBJS" => "Map objects",
        "ORDR" => "Orders",
        "ORDL" => "Order lists",
        "BKOR" => "Order backups",
        "OPTS" => "Settings (legacy)",
        "PATS" => "Settings",
        "SIGN" => "Signs",
        "STNS" => "Stations (legacy)",
        "STNN" => "Stations",
        "ROAD" => "Road stops",
        "PSAC" => "Persistent storage",
        "STPE" => "Story page elements",
        "STPA" => "Story pages",
        "NAME" => "Old names (legacy)",
        "SUBS" => "Subsidies",
        "CITY" => "Towns",
        "VEHS" => "Vehicles",
        "CHKP" => "Waypoints (legacy)",
        "ATID" => "_airporttile_mngr",
        "APID" => "_airport_mngr",
        "IIDS" => "_industry_mngr",
        "TIDS" => "_industile_mngr",
        "OBID" => "_object_mngr",
        "HIDS" => "_house_mngr",
        _ => panic!("Unknown chunk id: {}", chunk_id),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::loader::load_file;
    #[cfg(feature = "liblzma")]
    use crate::save_file::CompressedSaveFile;
    use crate::save_file::DebugSaveFile;

    #[test]
    fn test_load_valid_debug_file() {
        let file = File::open("./test-big.sav.decoded").unwrap();
        let save_file = DebugSaveFile::new_from_decoded(file);
        // let file = File::open("test_empty_map.sav").unwrap();
        // let save_file = CompressedSaveFile::new(file);

        let chunks = load_file(save_file).unwrap();

        // let mut out_file = File::create(Path::new("../out.json")).unwrap();
        // out_file
        //     .write(
        //         serde_json::to_string(
        //             chunks
        //                 .get("STNN")
        //                 .unwrap()
        //                 .iter()
        //                 .take(100)
        //                 .collect::<Vec<&TableItem>>()
        //                 .as_slice(),
        //         )
        //         .unwrap()
        //         .as_bytes(),
        //     )
        //     .unwrap();
    }

    #[test]
    #[cfg(feature = "liblzma")]
    fn test_load_valid_compressed_file() {
        let file = File::open("./test-big.sav").unwrap();
        let save_file = CompressedSaveFile::new(file);
        // let file = File::open("test_empty_map.sav").unwrap();
        // let save_file = CompressedSaveFile::new(file);

        let chunks = load_file(save_file).unwrap();
    }
}
