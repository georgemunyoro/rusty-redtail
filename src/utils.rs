pub fn get_bit(bitboard: u64, bit_index: u8) -> u64 {
    return (bitboard) & (1u64 << bit_index);
}

pub fn set_bit(bitboard: &mut u64, bit_index: u8) {
    *bitboard |= 1u64 << bit_index;
}

pub fn count_bits(bitboard: u64) -> u8 {
    let mut count = 0;
    let mut temp_bitboard = bitboard;
    while temp_bitboard != 0 {
        temp_bitboard &= temp_bitboard - 1;
        count += 1;
    }
    return count;
}

pub fn get_lsb(bitboard: u64) -> u8 {
    return bitboard.trailing_zeros() as u8;
}

pub fn pop_lsb(bitboard: &mut u64) -> u8 {
    let lsb = get_lsb(*bitboard);
    *bitboard &= *bitboard - 1;
    return lsb;
}

pub fn print_bitboard(bitboard: u64) {
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
