use std::fs::{File};
use std::io::prelude::*;
use std::path::Path;

pub fn load_file(path: &Path) {
    let mut file = File::open(path).unwrap();
    check_version_support(&mut file);
}

fn check_version_support(file: &mut File) {
    let mut version = [0; 4];

    file.read_exact(&mut version).unwrap();

    let version = std::str::from_utf8(&version).unwrap();
    if version != "OTTX" {
        panic!("Unsupported savegame version")
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