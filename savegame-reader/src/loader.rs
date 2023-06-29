use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use xz2::read::XzDecoder;

pub fn load_file(path: &Path) {
    let mut file = File::open(path).unwrap();
    check_version_support(&mut file);

    let mut decoder = XzDecoder::new(file);

    let mut b: Vec<u8> = vec![];
    decoder.read_to_end(&mut b).unwrap();
    unsafe { println!("{}", String::from_utf8_unchecked(b)) }
}

fn check_version_support(file: &mut impl Read) {
    let mut format = [0; 4];
    file.read_exact(&mut format).unwrap();
    let version = std::str::from_utf8(&format).unwrap();
    if version != "OTTX" {
        panic!("Unsupported savegame format")
    }

    let mut version = [0; 4];
    file.read_exact(&mut version).unwrap();
    let version = u32::from_be_bytes(version) >> 16;
    println!("Savegame version {}", version)
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