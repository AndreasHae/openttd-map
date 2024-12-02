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

fn chunk_name_of(chunk_id: &str) -> String {
    match chunk_id {
        "AIPL" => "AI companies".to_string(),
        "ANIT" => "Animated tiles".to_string(),
        "ERNW" => "Engine renewal".to_string(),
        "CMDL" => "Cargo deliveries".to_string(),
        "CMPU" => "Cargo pickups".to_string(),
        "CAPA" => "Cargo packets".to_string(),
        "CHTS" => "Cheats used".to_string(),
        "PLYR" => "Player companies".to_string(),
        "DEPT" => "Depots".to_string(),
        "PRIC" => "Prices".to_string(),
        "CAPR" => "Cargo payment rates (only in pre 126 savegames)".to_string(),
        "ECMY" => "Economy variables".to_string(),
        "CAPY" => "Cargo payments".to_string(),
        "ENGN" => "Engines".to_string(),
        "ENGS" => "Engine ID mappings (legacy)".to_string(),
        "EIDS" => "Engine ID mappings".to_string(),
        "GSDT" => "Game script configs".to_string(),
        "GSTR" => "Game strings".to_string(),
        "GLOG" => "Game log".to_string(),
        "GOAL" => "Goals".to_string(),
        "GRPS" => "Groups (of vehicles?)".to_string(),
        "INDY" => "Industries".to_string(),
        "IBLD" => "Industry builder".to_string(),
        "ITBL" => "Industry type build data".to_string(),
        "RAIL" => "Rail type labels".to_string(),
        "LEAE" => "League table elements".to_string(),
        "LEAT" => "League tables".to_string(),
        "LGRP" => "Link graphs".to_string(),
        "LGRJ" => "Link graph jobs".to_string(),
        "LGRS" => "Link graph schedule".to_string(),
        "MAPS" => "Map size".to_string(),
        "MAPT" => "Map tile types".to_string(),
        "MAPH" => "Map tile heights".to_string(),
        "MAPO" => "Map tile ownership information".to_string(),
        "MAP2" => "Map tile indices of towns, industries and stations".to_string(),
        "M3LO" => "Map tile general purpose".to_string(),
        "M3HI" => "Map tile general purpose".to_string(),
        "MAP5" => "Map tile general purpose".to_string(),
        "MAPE" => "Map tile general purpose".to_string(),
        "MAP7" => "Map tile NewGRF support".to_string(),
        "MAP8" => "Map tile general purpose".to_string(),
        "DATE" => "Ingame date".to_string(),
        "VIEW" => "Viewport info".to_string(),
        "NGRF" => "Loaded NewGRFs".to_string(),
        "OBJS" => "Map objects".to_string(),
        "ORDR" => "Orders".to_string(),
        "ORDL" => "Order lists".to_string(),
        "BKOR" => "Order backups".to_string(),
        "OPTS" => "Settings (legacy)".to_string(),
        "PATS" => "Settings".to_string(),
        "SIGN" => "Signs".to_string(),
        "STNS" => "Stations (legacy)".to_string(),
        "STNN" => "Stations".to_string(),
        "ROAD" => "Road stops".to_string(),
        "PSAC" => "Persistent storage".to_string(),
        "STPE" => "Story page elements".to_string(),
        "STPA" => "Story pages".to_string(),
        "NAME" => "Old names (legacy)".to_string(),
        "SUBS" => "Subsidies".to_string(),
        "CITY" => "Towns".to_string(),
        "VEHS" => "Vehicles".to_string(),
        "CHKP" => "Waypoints (legacy)".to_string(),
        "ATID" => "_airporttile_mngr".to_string(),
        "APID" => "_airport_mngr".to_string(),
        "IIDS" => "_industry_mngr".to_string(),
        "TIDS" => "_industile_mngr".to_string(),
        "OBID" => "_object_mngr".to_string(),
        "HIDS" => "_house_mngr".to_string(),
        unknown => format!("unknown chunk '{}'", unknown),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::loader::load_file;
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
    fn test_load_valid_compressed_file() {
        let file = File::open("./test-big.sav").unwrap();
        let save_file = CompressedSaveFile::new(file);
        // let file = File::open("test_empty_map.sav").unwrap();
        // let save_file = CompressedSaveFile::new(file);
        println!("Savefile Version: {}", save_file.version);

        let chunks = load_file(save_file).unwrap();
    }
}
