use std::io::Cursor;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::save_file::DebugSaveFile;
use crate::table_reader::TableItem;

mod common;
mod loader;
mod save_file;
mod table_reader;

#[wasm_bindgen]
pub fn load_file(buffer: &[u8]) -> String {
    console_error_panic_hook::set_once();

    let file = DebugSaveFile::new_from_decoded(Cursor::new(buffer));
    let chunks = loader::load_file(file).unwrap();
    let relevant_chunk_part = chunks
        .get("LGRP")
        .unwrap()
        .iter()
        .collect::<Vec<&TableItem>>();
    serde_json::to_string(relevant_chunk_part.as_slice()).unwrap()
}
