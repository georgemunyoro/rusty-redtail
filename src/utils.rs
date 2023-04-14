pub fn get_bit(bitboard: u64, bit_index: u8) -> u64 {
    return (bitboard) & (1 << bit_index);
}

pub fn set_bit(bitboard: u64, bit_index: u8) -> u64 {
    return bitboard | (1 << bit_index);
}

pub fn _print_bitboard(bitboard: u64) {
    for i in 0..64 {
        if i % 8 == 0 {
            println!();
        }
        print!(
            "{} ",
            match get_bit(bitboard, i) {
                0 => 0,
                _ => 1,
            }
        );
    }
    println!()
}
