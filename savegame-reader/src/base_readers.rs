use byteorder::ReadBytesExt;
use std::io::Read;
use std::mem::size_of;

pub fn read_conv(reader: &mut impl Read) {
    reader.read_u8().unwrap();
}

pub fn read_str(reader: &mut impl Read) -> String {
    let length = read_gamma(reader);
    let mut buf = vec![0; length];
    reader.read_exact(&mut buf).unwrap();

    String::from(std::str::from_utf8(&buf).unwrap())
}

pub fn has_bit(input: usize, n: u8) -> bool {
    if n > size_of::<usize>() as u8 {
        panic!(
            "Tried to check for bit at {} but usize is only {} bits long",
            n,
            size_of::<usize>()
        )
    }

    input & (1 << n) != 0
}

pub fn read_gamma(reader: &mut impl Read) -> usize {
    let mut length = usize::from(reader.read_u8().unwrap());
    if has_bit(length, 7) {
        length &= !0b1000_0000;
        if has_bit(length, 6) {
            length &= !0b0100_0000;
            if has_bit(length, 5) {
                length &= !0b0010_0000;
                if has_bit(length, 4) {
                    length &= !0b0001_0000;
                    if has_bit(length, 3) {
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

#[cfg(test)]
mod tests {
    use crate::base_readers::read_gamma;

    #[test]
    fn test_read_gamma() {
        assert_eq!(read_gamma(&mut [0b01000000].as_slice()), 64);
        assert_eq!(read_gamma(&mut [0b10100000, 0b00000000].as_slice()), 8192);
        assert_eq!(read_gamma(&mut [0b11010000, 0b00000000, 0b00000000].as_slice()), 1048576);
        assert_eq!(read_gamma(&mut [0b11101000, 0b00000000, 0b00000000, 0b00000000].as_slice()), 134217728);
        assert_eq!(read_gamma(&mut [0b11101000, 0b10000000, 0b10000000, 0b10000000].as_slice()), 142639232);
    }
}
