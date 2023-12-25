use std::mem::size_of;

pub fn has_bit(input: usize, n: u8) -> bool {
    let bits_in_type = size_of::<usize>() * 8;
    if n > bits_in_type as u8 {
        panic!(
            "Tried to check for bit at {} but usize is only {} bits long",
            n, bits_in_type
        )
    }

    input & (1 << n) != 0
}

#[cfg(test)]
mod tests {
    use crate::common::has_bit;

    #[test]
    fn test_has_bit() {
        assert_eq!(has_bit(0b1000_0000, 7), true);
        assert_eq!(has_bit(0b0100_0000, 6), true);
        assert_eq!(has_bit(0b0010_0000, 5), true);
        assert_eq!(has_bit(0b0001_0000, 4), true);
        assert_eq!(has_bit(0b0000_1000, 3), true);
        assert_eq!(has_bit(0b1111_0000, 3), false);
    }
}
